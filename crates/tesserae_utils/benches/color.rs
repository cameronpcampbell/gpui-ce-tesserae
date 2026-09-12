use std::{hint::black_box, time::Duration};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use palette::{Hsla, IntoColor, Oklaba, Srgba};
use tesserae_utils::PerceptualColor;

fn rgb(red: u8, green: u8, blue: u8) -> Oklaba {
    Srgba::new(
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
        1.0,
    )
    .into_color()
}

fn color(criterion: &mut Criterion) {
    let cases = [
        ("theme_background", rgb(2, 6, 23)),
        ("theme_foreground", rgb(229, 229, 229)),
        ("blue", rgb(106, 65, 255)),
        ("red", rgb(255, 0, 0)),
        ("green", rgb(0, 255, 0)),
        ("outside_srgb", Oklaba::new(0.55, 0.4, -0.4, 0.4)),
        ("near_black", Oklaba::new(0.0001, 0.0, 0.0, 1.0)),
        ("near_white", Oklaba::new(0.9999, 0.0, 0.0, 1.0)),
    ];

    let mut feedback = criterion.benchmark_group("feedback");
    for (name, color) in cases {
        for amount in [-0.08, -0.04, 0.04, 0.08] {
            feedback.bench_with_input(
                BenchmarkId::new(name, amount),
                &(color, amount),
                |b, &(color, amount)| {
                    b.iter(|| {
                        black_box(
                            black_box(color).perceptual_feedback(black_box(amount)),
                        )
                    });
                },
            );
        }
    }

    feedback.finish();

    let mut brightness = criterion.benchmark_group("brightness");
    for (name, color) in cases {
        for target in [0.01, 0.5, 0.99] {
            brightness.bench_with_input(
                BenchmarkId::new(name, target),
                &(color, target),
                |b, &(color, target)| {
                    b.iter(|| {
                        black_box(
                            black_box(color)
                                .perceptual_brightness(black_box(target)),
                        )
                    });
                },
            );
        }
    }

    brightness.finish();

    let blue = rgb(106, 65, 255);
    let srgb: Srgba = blue.into_color();
    let hsl: Hsla = blue.into_color();
    let mut representations = criterion.benchmark_group("representations");
    representations.bench_function("srgb_feedback", |b| {
        b.iter(|| black_box(black_box(srgb).perceptual_feedback(black_box(0.04))));
    });

    representations.bench_function("hsl_feedback", |b| {
        b.iter(|| black_box(black_box(hsl).perceptual_feedback(black_box(0.04))));
    });

    representations.bench_function("srgb_brightness", |b| {
        b.iter(|| black_box(black_box(srgb).perceptual_brightness(black_box(0.5))));
    });

    representations.bench_function("hsl_brightness", |b| {
        b.iter(|| black_box(black_box(hsl).perceptual_brightness(black_box(0.5))));
    });

    representations.finish();

    let mut early_returns = criterion.benchmark_group("early_returns");
    early_returns.bench_function("zero_feedback", |b| {
        b.iter(|| black_box(black_box(blue).perceptual_feedback(black_box(0.0))));
    });

    early_returns.bench_function("saturated_feedback", |b| {
        b.iter(|| black_box(black_box(blue).perceptual_feedback(black_box(1.0))));
    });

    early_returns.bench_function("brightness_endpoint", |b| {
        b.iter(|| black_box(black_box(blue).perceptual_brightness(black_box(1.0))));
    });

    // Match the displayed base used by `perceptual_brightness`.
    let displayed_rgb: Srgba = hsl.into_color();
    let displayed: Oklaba = displayed_rgb.into_color();
    early_returns.bench_function("unchanged_brightness", |b| {
        b.iter(|| {
            black_box(
                black_box(blue).perceptual_brightness(black_box(displayed.color.l)),
            )
        });
    });

    early_returns.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2))
        .sample_size(50);
    targets = color
}

criterion_main!(benches);
