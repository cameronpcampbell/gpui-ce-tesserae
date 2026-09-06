use palette::{
    Clamp, FromColor, Hsla, IntoColor, LinSrgba, Mix, Oklaba, Srgba, WithAlpha,
    color_difference::EuclideanDistance, convert::FromColorUnclamped,
};

// WCAG 2.1 flare offset, which keeps contrast finite near black.
const FLARE_LUMINANCE: f32 = 0.05;

// Parameters from P. Whittle, "Brightness, discriminability and the 'crispening
// effect'," Vision Research 32.8 (1992), doi:10.1016/0042-6989(92)90205-W.
const CONTRAST_GAIN: f32 = 6.58;
const DECREMENT_GAIN: f32 = 7.07 / 8.22;
const DISPLAY_TOLERANCE: f32 = 1e-7;

fn luminance(lightness: f32) -> f32 {
    lightness.clamp(0.0, 1.0).powi(3)
}

fn whittle_contrast(luminance: f32, surround: f32) -> f32 {
    (luminance - surround) / (luminance.min(surround) + FLARE_LUMINANCE)
}

fn whittle_brightness(contrast: f32) -> f32 {
    let gain = if contrast < 0.0 { DECREMENT_GAIN } else { 1.0 };

    gain * contrast.signum() * (1.0 + CONTRAST_GAIN * contrast.abs()).ln()
}

fn perceived_contrast(lightness: f32, surround_lightness: f32) -> f32 {
    whittle_brightness(whittle_contrast(
        luminance(lightness),
        luminance(surround_lightness),
    )) / whittle_brightness(whittle_contrast(1.0, 0.0))
}

/// Calculates the perceptual contrast between a color and its surround.
pub fn perceptual_contrast(
    color: impl IntoColor<Oklaba>,
    surround: impl IntoColor<Oklaba>,
) -> f32 {
    let color: Oklaba = color.into_color();
    let surround: Oklaba = surround.into_color();

    perceived_contrast(color.color.l, surround.color.l)
}

fn clipped_mix(base: Oklaba, endpoint: Oklaba, progress: f32) -> Oklaba {
    let candidate = base.mix(endpoint, progress);
    let rgb: LinSrgba = LinSrgba::from_color_unclamped(candidate);

    // Clipping keeps adjustments continuous near saturated blue.
    rgb.clamp().into_color()
}

fn gpui_displayed_color(color: Oklaba) -> Oklaba {
    // GPUI converts solid fills through HSL before displaying them.
    let hsl: Hsla = color.into_color();
    let rgb: Srgba = hsl.into_color();
    rgb.into_color()
}

#[inline(always)]
fn neutral_color(lightness: f32, alpha: f32) -> Oklaba {
    Oklaba::new(lightness, 0.0, 0.0, alpha)
}

/// Finds a clipped mix whose displayed measurement reaches `target`.
///
/// `measure` must return zero for `base` and increase toward `endpoint`.
fn search_displayed_mix(
    base: Oklaba,
    endpoint: Oklaba,
    target: f32,
    initial_progress: f32,
    tolerance: f32,
    measure: impl Fn(Oklaba) -> f32,
) -> Oklaba {
    let (mut lower, mut upper) = (0.0, 1.0);

    let mut progress = initial_progress.clamp(0.0, 1.0);
    let (mut previous_progress, mut previous_measured) = (0.0, 0.0);
    let mut best = base;
    let mut best_error = target;

    for _ in 0..24 {
        let candidate = clipped_mix(base, endpoint, progress);
        let measured = measure(gpui_displayed_color(candidate));
        let error = (measured - target).abs();

        if error < best_error {
            best = candidate;
            best_error = error;
        }

        if error <= tolerance {
            break;
        }

        if measured < target {
            lower = progress;
        } else {
            upper = progress;
        }

        // Estimate the target from the last two measurements. Flat measurements
        // or estimates outside the bracket fall back to bisection.
        let interpolated = progress
            + (target - measured) * (progress - previous_progress)
                / (measured - previous_measured);
        previous_progress = progress;
        previous_measured = measured;
        let next = if interpolated > lower && interpolated < upper {
            interpolated
        } else {
            (lower + upper) * 0.5
        };
        if next == progress {
            break;
        }
        progress = next;
    }

    best
}

fn set_displayed_lightness(color: Oklaba, target: f32) -> Oklaba {
    let base = gpui_displayed_color(color);
    if target == 0.0 || target == 1.0 {
        return neutral_color(target, base.alpha);
    }

    let lightness_delta = target - base.color.l;
    let target_delta = lightness_delta.abs();
    if target_delta <= DISPLAY_TOLERANCE {
        return base;
    }

    let direction = lightness_delta.signum();
    let endpoint =
        neutral_color(if direction > 0.0 { 1.0 } else { 0.0 }, base.alpha);

    search_displayed_mix(
        base,
        endpoint,
        target_delta,
        target_delta / (endpoint.color.l - base.color.l).abs(),
        DISPLAY_TOLERANCE,
        |displayed| direction * (displayed.color.l - base.color.l),
    )
}

/// Perceptual controls that adjust colors in OKLab and return the input type.
pub trait PerceptualColor: Sized {
    /// Returns the candidate with the greatest perceptual contrast against this color.
    fn best_contrast<C, const N: usize>(self, candidates: [C; N]) -> C
    where
        C: Clone + IntoColor<Oklaba>;

    /// Sets opacity with compensation based on the displayed color's lightness and chroma.
    fn perceptual_alpha(self, desired_alpha: f32) -> Self;

    /// Sets the color's displayed lightness while preserving its opacity.
    fn perceptual_brightness(self, intensity: f32) -> Self;

    /// Moves the color toward white or black by the requested distance in OKLab.
    fn perceptual_feedback(self, amount: f32) -> Self;
}

impl<C> PerceptualColor for C
where
    C: Clone + IntoColor<Oklaba> + FromColor<Oklaba> + WithAlpha<f32, WithAlpha = C>,
{
    fn best_contrast<T, const N: usize>(self, candidates: [T; N]) -> T
    where
        T: Clone + IntoColor<Oklaba>,
    {
        let base: Oklaba = self.into_color();
        let mut candidates = candidates.into_iter();
        let first = candidates
            .next()
            .expect("best_contrast requires at least one candidate");

        let contrast = |candidate: &T| {
            let candidate: Oklaba = candidate.clone().into_color();
            perceived_contrast(candidate.color.l, base.color.l).abs()
        };
        let first_contrast = contrast(&first);

        candidates
            .fold((first, first_contrast), |best, candidate| {
                let candidate_contrast = contrast(&candidate);
                if candidate_contrast > best.1 {
                    (candidate, candidate_contrast)
                } else {
                    best
                }
            })
            .0
    }

    fn perceptual_alpha(self, desired_alpha: f32) -> Self {
        let alpha = desired_alpha.clamp(0.0, 1.0);
        if alpha == 0.0 || alpha == 1.0 {
            return self.with_alpha(alpha);
        }

        let color = gpui_displayed_color(self.clone().into_color()).color;
        let lightness = color.l.clamp(0.0, 1.0);
        let chroma = color.a.hypot(color.b);
        let magnitude = lightness.hypot(chroma).clamp(0.0, 1.0);

        self.with_alpha(alpha + (1.0 - magnitude) * alpha * (1.0 - alpha))
    }

    fn perceptual_brightness(self, intensity: f32) -> Self {
        set_displayed_lightness(self.into_color(), intensity.clamp(0.0, 1.0))
            .into_color()
    }

    fn perceptual_feedback(self, amount: f32) -> Self {
        if amount == 0.0 {
            return self;
        }

        let base = gpui_displayed_color(self.into_color());
        let endpoint =
            neutral_color(if amount > 0.0 { 1.0 } else { 0.0 }, base.alpha);
        let target = amount.abs().min(1.0);
        let available = base.color.distance(endpoint.color);
        if target >= available {
            return endpoint.into_color();
        }

        search_displayed_mix(
            base,
            endpoint,
            target,
            target / available,
            (target * 1e-5).max(DISPLAY_TOLERANCE),
            |displayed| base.color.distance(displayed.color),
        )
        .into_color()
    }
}

#[cfg(test)]
mod tests {
    use gpui::Background;
    use palette::{
        IsWithinBounds, LinSrgb, Oklaba, Srgba, color_difference::EuclideanDistance,
    };

    use super::*;

    const BLACK: Oklaba = Oklaba::new(0.0, 0.0, 0.0, 1.0);
    const WHITE: Oklaba = Oklaba::new(1.0, 0.0, 0.0, 1.0);

    fn assert_close(actual: f32, expected: f32) {
        assert_close_within(actual, expected, 1e-6);
    }

    fn assert_close_within(actual: f32, expected: f32, tolerance: f32) {
        assert!(
            (actual - expected).abs() < tolerance,
            "{actual} is not within {tolerance} of {expected}"
        );
    }

    fn gray(lightness: f32) -> Oklaba {
        Oklaba::new(lightness, 0.0, 0.0, 1.0)
    }

    #[test]
    fn alpha_replaces_opacity_and_preserves_color_across_representations() {
        fn check_alpha<C>(original: C, desired: f32) -> f32
        where
            C: Copy + PerceptualColor + WithAlpha<f32, WithAlpha = C>,
            C::Color: std::fmt::Debug + PartialEq,
        {
            let expected = original.perceptual_alpha(desired).split();
            assert_eq!(expected.0, original.without_alpha());
            for initial_alpha in [0.0, 0.4, 1.0, expected.1] {
                assert_eq!(
                    original
                        .with_alpha(initial_alpha)
                        .perceptual_alpha(desired)
                        .split(),
                    expected,
                );
            }
            expected.1
        }

        for sample in color_samples() {
            let hsl = Background::from(sample).as_solid().unwrap();
            let srgb: Srgba = hsl.into_color();
            for desired in [-2.0, 0.0, 1e-6, 0.08, 0.2, 0.5, 0.9, 1.0, 2.0] {
                let alpha = check_alpha(sample, desired);
                assert_close_within(check_alpha(srgb, desired), alpha, 1e-5);
                assert_close_within(check_alpha(hsl, desired), alpha, 1e-5);
            }
        }
    }

    #[test]
    fn alpha_increases_smoothly_with_bounded_compensation() {
        for color in color_samples() {
            let mut previous_alpha = 0.0;
            let mut previous_desired = 0.0;
            let requests = [-2.0_f32, 0.0, 1e-8, 1e-7, 1e-6, 1e-5, 1e-4, 1e-3]
                .into_iter()
                .chain((1..100).map(|step| step as f32 / 100.0))
                .chain([0.9999, 0.99999, 0.999999, 1.0, 2.0]);

            for request in requests {
                let alpha = color.perceptual_alpha(request).alpha;
                let desired = request.clamp(0.0, 1.0);
                assert!(
                    (desired..=(2.0 * desired).min(1.0)).contains(&alpha),
                    "base {color:?}, desired {desired}: alpha {alpha}",
                );
                assert!(alpha >= previous_alpha);
                assert!(
                    alpha - previous_alpha
                        <= 2.0 * (desired - previous_desired) + 2.0 * f32::EPSILON,
                    "base {color:?}, desired {desired}: abrupt alpha change",
                );
                previous_alpha = alpha;
                previous_desired = desired;
            }
        }
    }

    #[test]
    fn alpha_compensation_decreases_with_lightness_and_chroma() {
        for desired in [0.08, 0.2, 0.5, 0.9] {
            let mut previous_alpha = BLACK.perceptual_alpha(desired).alpha;
            for step in 1..=100 {
                let alpha =
                    gray(step as f32 / 100.0).perceptual_alpha(desired).alpha;
                assert!(alpha < previous_alpha);
                assert!(previous_alpha - alpha < 0.01);
                previous_alpha = alpha;
            }
            assert_close(previous_alpha, desired);

            let neutral = gray(0.5).perceptual_alpha(desired);
            let colorful = Oklaba::new(0.5, 0.1, 0.0, 1.0).perceptual_alpha(desired);
            assert!(colorful.alpha < neutral.alpha);
            assert!(colorful.alpha > desired);
        }
    }

    #[test]
    fn contrast_is_signed_normalized_and_ordered_by_lightness() {
        assert_close(perceptual_contrast(WHITE, BLACK), 1.0);
        assert!((-1.0..0.0).contains(&perceptual_contrast(BLACK, WHITE)));

        for surround_lightness in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let surround = gray(surround_lightness);
            let contrasts = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
                .map(|lightness| perceptual_contrast(gray(lightness), surround));

            assert!(contrasts.windows(2).all(|pair| pair[0] < pair[1]));
            assert_close(perceptual_contrast(surround, surround), 0.0);
            for lightness in [0.1, 0.4, 0.7, 0.9] {
                let contrast = perceptual_contrast(gray(lightness), surround);

                if lightness < surround_lightness {
                    assert!(contrast < 0.0);
                } else if lightness > surround_lightness {
                    assert!(contrast > 0.0);
                }
            }
        }

        let chromatic = Oklaba::new(0.6, 0.3, -0.2, 0.2);
        assert_close(
            perceptual_contrast(chromatic, gray(0.3)),
            perceptual_contrast(gray(0.6), gray(0.3)),
        );
    }

    #[test]
    fn best_contrast_returns_the_farthest_candidate_and_keeps_its_type() {
        assert_eq!(gray(0.2).best_contrast([gray(0.1), WHITE]), WHITE);
        assert_eq!(gray(0.8).best_contrast([BLACK, gray(0.9)]), BLACK);
        let first = Srgba::new(0.8, 0.4, 0.2, 0.3);
        let second = Srgba::new(0.8, 0.4, 0.2, 0.9);
        assert_eq!(BLACK.best_contrast([first, second]), first);
    }

    #[test]
    #[should_panic(expected = "best_contrast requires at least one candidate")]
    fn best_contrast_rejects_an_empty_candidate_array() {
        let _: Oklaba = BLACK.best_contrast([]);
    }

    fn srgb_samples() -> impl Iterator<Item = Srgba> {
        let channels = [0.0, 0.25, 0.5, 0.75, 1.0];
        // Exercise colors between grid points without a random test dependency.
        let interior = (0..64).scan(0x5eed_u32, |state, _| {
            let [red, green, blue] = std::array::from_fn(|_| {
                *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                (*state >> 8) as f32 / (1_u32 << 24) as f32
            });
            Some(Srgba::new(red, green, blue, 1.0))
        });
        let saturated: [Srgba; 4] = [
            Srgba::new(1.0, 0.0, 0.0, 1.0),
            Srgba::new(0.0, 1.0, 0.0, 1.0),
            Srgba::new(0.0, 0.0, 1.0, 1.0),
            Srgba::new(106.0 / 255.0, 65.0 / 255.0, 1.0, 1.0),
        ];
        channels
            .into_iter()
            .flat_map(move |red| {
                channels.into_iter().flat_map(move |green| {
                    channels
                        .into_iter()
                        .map(move |blue| Srgba::new(red, green, blue, 1.0))
                })
            })
            .chain(interior)
            .chain(saturated.into_iter().flat_map(|color| {
                [0.0, 1e-6, 1e-4].map(|offset| {
                    Srgba::new(
                        color.red.clamp(offset, 1.0 - offset),
                        color.green.clamp(offset, 1.0 - offset),
                        color.blue.clamp(offset, 1.0 - offset),
                        color.alpha,
                    )
                })
            }))
    }

    fn displayed_color(color: impl IntoColor<Hsla>) -> Oklaba {
        let rgb: Srgba = Background::from(color).as_solid().unwrap().into_color();
        rgb.into_color()
    }

    fn color_samples() -> impl Iterator<Item = Oklaba> {
        srgb_samples().map(|color| color.into_color()).chain([
            gray(0.0001),
            gray(0.9999),
            // Start from the displayed color even when the input exceeds sRGB.
            Oklaba::new(0.55, 0.4, -0.4, 0.4),
            Oklaba::new(0.72, -0.3, 0.3, 0.6),
        ])
    }

    fn chroma(color: Oklaba) -> f32 {
        color.color.a.hypot(color.color.b)
    }

    fn hue_difference(left: Oklaba, right: Oklaba) -> f32 {
        let difference = (left.color.b.atan2(left.color.a)
            - right.color.b.atan2(right.color.a))
        .to_degrees();

        (difference + 180.0).rem_euclid(360.0) - 180.0
    }

    fn assert_in_gamut(color: Oklaba) {
        let rgb: LinSrgb = LinSrgb::from_color_unclamped(color.color);
        assert!(
            [rgb.red, rgb.green, rgb.blue]
                .into_iter()
                .all(|channel| (-1e-5..=1.0 + 1e-5).contains(&channel))
        );
    }

    fn assert_equivalent(alpha: f32, oklab: Oklaba, srgb: Srgba, hsl: Hsla) {
        assert_eq!([oklab.alpha, srgb.alpha, hsl.alpha], [alpha; 3]);
        assert!(srgb.is_within_bounds());
        assert!(hsl.is_within_bounds());

        let displayed = displayed_color(oklab);
        for other in [displayed_color(srgb), displayed_color(hsl)] {
            assert!(displayed.color.distance(other.color) < 1e-5);
        }
    }

    #[test]
    fn brightness_reaches_displayed_targets_in_order_with_neutral_limits() {
        for color in color_samples() {
            let original_chroma = chroma(displayed_color(color));
            let mut previous_lightness = 0.0;
            let intensities = [-2.0, -1.0, 0.0, 1e-7, 1e-6, 1e-5, 1e-4]
                .into_iter()
                .chain((1..100).map(|step| step as f32 / 100.0))
                .chain([0.9999, 0.99999, 0.999999, 1.0, 2.0]);

            for intensity in intensities {
                let adjusted = color.perceptual_brightness(intensity);
                let displayed = displayed_color(adjusted);
                let lightness = displayed.color.l;
                assert!(
                    (lightness - intensity.clamp(0.0, 1.0)).abs() < 2e-6,
                    "base {color:?}, intensity {intensity}: lightness {lightness}",
                );
                assert!(lightness >= previous_lightness - 1e-6);
                assert_eq!(adjusted.alpha, color.alpha);
                assert!(chroma(displayed) <= original_chroma + 1e-5);
                assert_in_gamut(adjusted);
                if intensity <= 0.0 {
                    assert_eq!(adjusted.color, BLACK.color);
                } else if intensity >= 1.0 {
                    assert_eq!(adjusted.color, WHITE.color);
                }
                previous_lightness = lightness;
            }
        }
    }

    #[test]
    fn brightness_is_stable_near_the_original_color() {
        for color in color_samples() {
            let rest = displayed_color(color);
            let original_chroma = chroma(rest);
            assert!(
                rest.color.distance(
                    displayed_color(color.perceptual_brightness(rest.color.l)).color
                ) < 1e-5
            );

            for delta in [-1e-5_f32, -2e-7, -1e-7, -5e-8, 5e-8, 1e-7, 2e-7, 1e-5] {
                let target = (rest.color.l + delta).clamp(0.0, 1.0);
                let adjusted = displayed_color(color.perceptual_brightness(target));
                assert!(
                    (adjusted.color.l - target).abs() < 2e-6,
                    "base {color:?}, target {target}: displayed {adjusted:?}",
                );
                assert!(rest.color.distance(adjusted.color) < 0.01);
                assert!(chroma(adjusted) <= original_chroma + 1e-5);

                if original_chroma > 0.05 && chroma(adjusted) > 0.05 {
                    assert!(hue_difference(adjusted, rest).abs() < 2.0);
                }
            }
        }
    }

    #[test]
    fn feedback_changes_smoothly_in_both_directions_until_the_endpoint() {
        for color in color_samples() {
            assert_eq!(color.perceptual_feedback(0.0), color);
            let rest = displayed_color(color);
            let original_chroma = chroma(rest);
            for direction in [-1.0, 1.0] {
                let endpoint = if direction > 0.0 { WHITE } else { BLACK };
                let available = rest.color.distance(displayed_color(endpoint).color);
                let mut previous = rest;
                let mut previous_amount = 0.0;
                let mut amounts: Vec<_> = [1e-9, 1e-8, 5e-8, 1e-7, 1e-6, 1e-5, 1e-4]
                    .into_iter()
                    .chain((1..=100).map(|step| step as f32 / 100.0))
                    .chain([
                        (available - 1e-6).max(0.0),
                        available.next_down().max(0.0),
                        available,
                        available.next_up(),
                        available + 1e-6,
                        2.0,
                    ])
                    .collect();
                amounts.sort_by(f32::total_cmp);
                amounts.dedup();

                for amount in amounts {
                    let adjusted = color.perceptual_feedback(direction * amount);
                    let displayed = displayed_color(adjusted);
                    let strength = rest.color.distance(displayed.color);
                    let tolerance = if amount <= 1e-4 { 2e-6 } else { 1e-4 };
                    assert!(
                        (strength - amount.min(available)).abs() < tolerance,
                        "base {color:?}, amount {}: distance {strength}",
                        direction * amount,
                    );
                    assert!(
                        (displayed.color.l - previous.color.l) * direction >= -1e-6
                    );
                    assert!(
                        previous.color.distance(displayed.color)
                            <= (amount - previous_amount) * 1.5 + 2e-6,
                        "base {color:?}, amount {}: discontinuous feedback",
                        direction * amount,
                    );
                    assert_eq!(adjusted.alpha, color.alpha);
                    assert_in_gamut(adjusted);
                    if amount >= available {
                        assert!(displayed.color.distance(endpoint.color) < 1e-6);
                    }

                    let adjusted_chroma = chroma(displayed);
                    assert!(adjusted_chroma <= original_chroma + 1e-5);
                    if original_chroma < 1e-5 {
                        assert!(adjusted_chroma < 1e-5);
                    } else if amount <= 0.08
                        && original_chroma > 0.05
                        && adjusted_chroma > 0.05
                    {
                        let hue_change = hue_difference(displayed, rest);
                        assert!(
                            hue_change.abs() < 2.0,
                            "base {color:?}, amount {}: hue change {hue_change}",
                            direction * amount,
                        );
                    }

                    previous = displayed;
                    previous_amount = amount;
                }
            }
        }
    }

    #[test]
    fn adjustments_preserve_alpha_gamut_and_representation() {
        for sample in color_samples() {
            for alpha in [0.0, 0.4, 1.0] {
                let original = sample.with_alpha(alpha);
                let hsl = Background::from(original).as_solid().unwrap();
                let srgb: Srgba = hsl.into_color();

                assert_eq!(original.perceptual_feedback(0.0), original);
                assert_eq!(srgb.perceptual_feedback(0.0), srgb);
                assert_eq!(hsl.perceptual_feedback(0.0), hsl);

                for intensity in [0.0, 1e-6, 0.2, 0.5, 0.8, 0.999999, 1.0] {
                    assert_equivalent(
                        alpha,
                        original.perceptual_brightness(intensity),
                        srgb.perceptual_brightness(intensity),
                        hsl.perceptual_brightness(intensity),
                    );
                }

                for amount in [-1.0_f32, -0.08, -0.04, -1e-7, 1e-7, 0.04, 0.08, 1.0]
                {
                    assert_equivalent(
                        alpha,
                        original.perceptual_feedback(amount),
                        srgb.perceptual_feedback(amount),
                        hsl.perceptual_feedback(amount),
                    );
                }
            }
        }
    }
}
