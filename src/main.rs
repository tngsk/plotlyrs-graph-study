use plotly::{
    common::{Font, Line, Marker, Mode, Title},
    layout::{Annotation, Axis, GridPattern, LayoutGrid, Margin},
    ImageFormat, Layout, Plot, Scatter,
};
use std::f64::consts::PI;

#[derive(Clone)]
struct SignalParams {
    name: String,
    signal_freq: f64,   // 入力信号の周波数 (Hz)
    sampling_rate: i64, // サンプリング周波数 (Hz)
    bit_depth: u32,     // 量子化ビット数
    nyquist_ratio: f64, // ナイキスト周波数との比率
}

impl SignalParams {
    fn new(name: &str, signal_freq: f64, sampling_rate: i64, bit_depth: u32) -> Self {
        let nyquist_ratio = (2.0 * signal_freq) / (sampling_rate as f64);
        Self {
            name: name.to_string(),
            signal_freq,
            sampling_rate,
            bit_depth,
            nyquist_ratio,
        }
    }
}

fn create_layout_guides() -> Vec<Annotation> {
    let mut guides = Vec::new();

    // ガイド用フォントの設定
    let guide_font = Font::new().size(8).color("#999999").family("Fira Code");

    // X軸ガイド（0.0から1.0まで0.1刻み）
    for i in 0..=10 {
        let x = i as f64 * 0.1;
        guides.push(
            Annotation::new()
                .text(&format!("x: {:.1}", x))
                .x_ref("paper")
                .y_ref("paper")
                .x(x)
                .y(0.0) // 下端
                .show_arrow(false)
                .font(guide_font.clone()),
        );

        // 垂直の点線を示すためのテキスト
        guides.push(
            Annotation::new()
                .text("|")
                .x_ref("paper")
                .y_ref("paper")
                .x(x)
                .y(0.5) // 中央
                .show_arrow(false)
                .font(guide_font.clone()),
        );
    }

    // Y軸ガイド（0.0から1.0まで0.1刻み）
    for i in 0..=10 {
        let y = i as f64 * 0.1;
        guides.push(
            Annotation::new()
                .text(&format!("y: {:.1}", y))
                .x_ref("paper")
                .y_ref("paper")
                .x(0.0) // 左端
                .y(y)
                .show_arrow(false)
                .font(guide_font.clone()),
        );

        // 水平の点線を示すためのテキスト
        guides.push(
            Annotation::new()
                .text("—")
                .x_ref("paper")
                .y_ref("paper")
                .x(0.5) // 中央
                .y(y)
                .show_arrow(false)
                .font(guide_font.clone()),
        );
    }

    guides
}

fn create_sine_wave(params: &SignalParams) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let dt = 1.0 / params.sampling_rate as f64; // サンプリング間隔
    let time_range = 2.0;
    let num_samples = (time_range * params.sampling_rate as f64) as i64;

    // 連続信号の表示用パラメータ
    const INTERPOLATION_FACTOR: i64 = 20; // 補間の細かさ（1サンプル間を何分割するか）オーバーサンプリング率

    // 理想的な連続信号（高解像度でプロット）
    let continuous_points = num_samples * INTERPOLATION_FACTOR; // 表示用の総ポイント数
    let continuous_x: Vec<f64> = (0..continuous_points)
        .map(|i| i as f64 * dt / INTERPOLATION_FACTOR as f64) // より細かい時間間隔
        .take_while(|&t| t <= time_range)
        .collect();

    // 減衰係数（時定数）
    let decay_rate = 0.5; // 減衰の速さを調整（大きいほど速く減衰）

    let continuous_y: Vec<f64> = continuous_x
        .iter()
        .map(|&t| {
            let decay = (-decay_rate * t).exp(); // 指数減衰
            decay * (2.0 * PI * params.signal_freq * t).sin()
        })
        .collect();

    // サンプリングと量子化
    let amplitude_levels = 2u32.pow(params.bit_depth) as f64;

    let sample_x: Vec<f64> = (0..num_samples).map(|i| i as f64 * dt).collect();

    let sample_y: Vec<f64> = sample_x
        .iter()
        .map(|&t| {
            let decay = (-decay_rate * t).exp(); // 指数減衰
            let raw_sin = decay * (2.0 * PI * params.signal_freq * t).sin();
            (raw_sin * amplitude_levels / 2.0).round() / (amplitude_levels / 2.0)
        })
        .collect();

    (continuous_x, continuous_y, sample_x, sample_y)
}

fn generate_title(params: &SignalParams) -> String {
    format!(
        "{} (Nyquist Ratio: {:.2})<br>Signal: {:.1}Hz<br>Sampling: {}Hz<br>Bit Depth: {}-bit",
        params.name,
        params.nyquist_ratio,
        params.signal_freq,
        params.sampling_rate,
        params.bit_depth
    )
}

fn main() {
    let mut plot = Plot::new();

    // エイリアシングを示すパラメータセット
    let params = vec![
        SignalParams::new("Severe Aliasing", 10.0, 8, 16),
        SignalParams::new("Aliasing", 10.0, 12, 16),
        SignalParams::new("Near Nyquist", 10.0, 24, 16),
        SignalParams::new("Hi Resolution", 10.0, 240, 16),
    ];

    // サブプロットの作成
    for (i, param) in params.iter().enumerate() {
        let (cont_x, cont_y, sample_x, sample_y) = create_sine_wave(param);

        // 理想的な連続信号（オリジナル）
        let continuous = Scatter::new(cont_x, cont_y)
            .name("Original Signal")
            .mode(Mode::Lines)
            .line(Line::new().color("rgba(170, 170, 170, 0.5)"))
            .x_axis(format!("x{}", i + 1))
            .y_axis(format!("y{}", i + 1));

        // サンプリング点と再構成信号
        let samples = Scatter::new(sample_x, sample_y)
            .name("Sampled & Reconstructed")
            .mode(Mode::LinesMarkers)
            .line(Line::new().color("rgba(31, 119, 180, 1.0)"))
            .marker(Marker::new().size(8).color("rgba(255, 0, 0, 0.7)")) // サンプリング点を赤で強調
            .x_axis(format!("x{}", i + 1))
            .y_axis(format!("y{}", i + 1));

        plot.add_trace(continuous);
        plot.add_trace(samples);
    }

    // レイアウト設定

    let subplot_title_font = Font::new().size(8).color("#333").family("Fira Code");
    let axis_font = Font::new().size(7).color("#333").family("Fira Code");
    let tick_font = Font::new().size(6).color("#333").family("Fira Code");
    let mut layout = Layout::new()
        .margin(
            Margin::new()
                .left(0)
                .right(0)
                .top(0)
                .bottom(0)
                .pad(0)
                .auto_expand(true),
        )
        .grid(
            LayoutGrid::new()
                .rows(2)
                .columns(2)
                .pattern(GridPattern::Independent)
                .sub_plots(vec!["subplot".to_string()]),
        )
        .show_legend(false)
        .x_axis(
            Axis::new()
                .title(Title::with_text("Time (s)").font(axis_font.clone()))
                .domain(&[0.05, 0.45])
                // .range(vec![0.0, 1.0]) // 0-1秒に固定
                .tick_font(tick_font.clone()),
        )
        .y_axis(
            Axis::new()
                .title(Title::with_text("Amplitude").font(axis_font.clone()))
                .domain(&[0.55, 0.95])
                .tick_font(tick_font.clone())
                .range(vec![-1.2, 1.2]),
        )
        .x_axis2(
            Axis::new()
                .title(Title::with_text("Time (s)").font(axis_font.clone()))
                .domain(&[0.55, 0.95])
                // .range(vec![0.0, 1.0]) // 0-1秒に固定
                .tick_font(tick_font.clone()),
        )
        .y_axis2(
            Axis::new()
                .title(Title::with_text("Amplitude").font(axis_font.clone()))
                .domain(&[0.55, 0.95])
                .tick_font(tick_font.clone())
                .range(vec![-1.2, 1.2]),
        )
        .x_axis3(
            Axis::new()
                .title(Title::with_text("Time (s)").font(axis_font.clone()))
                .domain(&[0.05, 0.45])
                // .range(vec![0.0, 1.0]) // 0-1秒に固定
                .tick_font(tick_font.clone()),
        )
        .y_axis3(
            Axis::new()
                .title(Title::with_text("Amplitude").font(axis_font.clone()))
                .domain(&[0.05, 0.45])
                .tick_font(tick_font.clone())
                .range(vec![-1.2, 1.2]),
        )
        .x_axis4(
            Axis::new()
                .title(Title::with_text("Time (s)").font(axis_font.clone()))
                .domain(&[0.55, 0.95])
                // .range(vec![0.0, 1.0]) // 0-1秒に固定
                .tick_font(tick_font.clone()),
        )
        .y_axis4(
            Axis::new()
                .title(Title::with_text("Amplitude").font(axis_font.clone()))
                .domain(&[0.05, 0.45])
                .tick_font(tick_font.clone())
                .range(vec![-1.2, 1.2]),
        );

    // サブプロットのタイトル用のアノテーションを設定
    let annotations = vec![
        Annotation::new()
            .show_arrow(false)
            .text(generate_title(&params[0]))
            .font(subplot_title_font.clone())
            .align(plotly::layout::HAlign::Left)
            .valign(plotly::layout::VAlign::Top)
            .x_anchor(plotly::common::Anchor::Right)
            .y_anchor(plotly::common::Anchor::Top)
            .x_ref("x")
            .y_ref("y")
            .x(1.95)
            .y(0.95)
            .border_color("#333")
            .border_pad(2.0)
            .background_color("#fff"),
        Annotation::new()
            .show_arrow(false)
            .text(generate_title(&params[1]))
            .font(subplot_title_font.clone())
            .align(plotly::layout::HAlign::Left)
            .valign(plotly::layout::VAlign::Top)
            .x_anchor(plotly::common::Anchor::Right)
            .y_anchor(plotly::common::Anchor::Top)
            .x_ref("x2")
            .y_ref("y2")
            .x(1.95)
            .y(0.95)
            .border_color("#333")
            .border_pad(2.0)
            .background_color("#fff"),
        Annotation::new()
            .show_arrow(false)
            .text(generate_title(&params[2]))
            .font(subplot_title_font.clone())
            .align(plotly::layout::HAlign::Left)
            .valign(plotly::layout::VAlign::Top)
            .x_anchor(plotly::common::Anchor::Right)
            .y_anchor(plotly::common::Anchor::Top)
            .x_ref("x3")
            .y_ref("y3")
            .x(1.95)
            .y(0.95)
            .border_color("#333")
            .border_pad(2.0)
            .background_color("#fff"),
        Annotation::new()
            .show_arrow(false)
            .text(generate_title(&params[3]))
            .font(subplot_title_font.clone())
            .align(plotly::layout::HAlign::Left)
            .valign(plotly::layout::VAlign::Top)
            .x_anchor(plotly::common::Anchor::Right)
            .y_anchor(plotly::common::Anchor::Top)
            .x_ref("x4")
            .y_ref("y4")
            .x(1.95)
            .y(0.95)
            .border_color("#333")
            .border_pad(2.0)
            .background_color("#fff"),
    ];

    for annotation in annotations {
        layout.add_annotation(annotation);
    }

    // layout guides
    for annotation in create_layout_guides() {
        layout.add_annotation(annotation);
    }

    plot.set_layout(layout);
    plot.write_image(
        "export/digital_audio_comparison.png",
        ImageFormat::PNG,
        1200,
        800,
        4.0,
    );
}
