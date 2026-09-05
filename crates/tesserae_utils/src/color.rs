use palette::{
    FromColor, IntoColor, IsWithinBounds, LinSrgb, Oklaba, WithAlpha,
    convert::FromColorUnclamped,
};

// WCAG 2.1 flare offset, which keeps contrast finite near black.
const FLARE_LUMINANCE: f32 = 0.05;

// Parameters from P. Whittle, "Brightness, discriminability and the 'crispening
// effect'," Vision Research 32.8 (1992), doi:10.1016/0042-6989(92)90205-W.
const CONTRAST_GAIN: f32 = 6.58;
const DECREMENT_GAIN: f32 = 7.07 / 8.22;

fn perceptual_magnitude(color: &Oklaba) -> f32 {
    let lightness = color.color.l.clamp(0.0, 1.0);
    let chroma = color.color.a.hypot(color.color.b);

    lightness.hypot(chroma).clamp(0.0, 1.0)
}

fn remap_perceptual_value(value: f32, magnitude: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);

    if value == 0.0 || value == 1.0 {
        value
    } else {
        value.powf(magnitude)
    }
}

fn luminance(lightness: f32) -> f32 {
    lightness.clamp(0.0, 1.0).powi(3)
}

fn whittle_contrast(luminance: f32, surround: f32) -> f32 {
    (luminance - surround) / (luminance.min(surround) + FLARE_LUMINANCE)
}

fn brightness(contrast: f32) -> f32 {
    let gain = if contrast < 0.0 { DECREMENT_GAIN } else { 1.0 };

    gain * contrast.signum() * (1.0 + CONTRAST_GAIN * contrast.abs()).ln()
}

fn brightness_range() -> f32 {
    brightness(whittle_contrast(1.0, 0.0))
}

fn perceived_contrast(lightness: f32, surround_lightness: f32) -> f32 {
    brightness(whittle_contrast(
        luminance(lightness),
        luminance(surround_lightness),
    )) / brightness_range()
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

fn fit_feedback_to_srgb(mut color: Oklaba) -> Oklaba {
    if color.color.l == 0.0 || color.color.l == 1.0 {
        color.color.a = 0.0;
        color.color.b = 0.0;
        return color;
    }

    let in_gamut = |color: Oklaba| {
        let rgb: LinSrgb = LinSrgb::from_color_unclamped(color.color);
        rgb.is_within_bounds()
    };
    if in_gamut(color) {
        return color;
    }

    // Reduce chroma at fixed lightness and hue so GPUI's HSL conversion
    // does not clamp the color and change the requested lightness step.
    let original = color;
    let (mut low, mut high) = (0.0, 1.0);
    // Twenty bisections resolve the chroma scale to within 1e-6.
    for _ in 0..20 {
        let scale = (low + high) * 0.5;
        color.color.a = original.color.a * scale;
        color.color.b = original.color.b * scale;
        if in_gamut(color) {
            low = scale;
        } else {
            high = scale;
        }
    }

    color.color.a = original.color.a * low;
    color.color.b = original.color.b * low;
    color
}

/// Perceptual controls that adjust colors in OKLab and return the input type.
pub trait PerceptualColor: Sized {
    /// Returns the candidate with the greatest perceptual contrast against this color.
    ///
    /// When multiple candidates have the same contrast, this returns the first one.
    ///
    /// # Panics
    ///
    /// Panics if `candidates` is empty.
    fn best_contrast<C, const N: usize>(self, candidates: [C; N]) -> C
    where
        C: Clone + IntoColor<Oklaba>;

    /// Applies a perceptual alpha to the color using its lightness and chroma.
    fn perceptual_alpha(self, desired_alpha: f32) -> Self;

    /// Applies a perceptual brightness to the color using its lightness and chroma.
    fn perceptual_brightness(self, intensity: f32) -> Self;

    /// Adds a signed OKLab lightness step, clamped at black and white.
    ///
    /// Positive amounts brighten; negative amounts darken. Zero returns the
    /// original color exactly. The amount must be finite and is clamped to
    /// `[-1.0, 1.0]`.
    ///
    /// Preserves alpha and hue, reducing chroma only as needed to fit sRGB.
    /// Black and white have zero chroma. Equal steps target equal lightness
    /// changes for opaque colors; alpha is not compensated for compositing.
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
        let perceptual_color = self.clone().into_color();
        let magnitude = perceptual_magnitude(&perceptual_color);
        let alpha = remap_perceptual_value(desired_alpha, magnitude);

        self.with_alpha(alpha)
    }

    fn perceptual_brightness(self, intensity: f32) -> Self {
        let mut perceptual_color = self.into_color();
        let magnitude = perceptual_magnitude(&perceptual_color);

        perceptual_color.color.l = remap_perceptual_value(intensity, magnitude);
        perceptual_color.into_color()
    }

    fn perceptual_feedback(self, amount: f32) -> Self {
        if amount == 0.0 {
            return self;
        }

        let mut perceptual_color: Oklaba = self.into_color();

        perceptual_color.color.l =
            (perceptual_color.color.l + amount.clamp(-1.0, 1.0)).clamp(0.0, 1.0);
        fit_feedback_to_srgb(perceptual_color).into_color()
    }
}

#[cfg(test)]
mod tests {
    use palette::{Hsla, Oklaba, Srgba};

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
    fn absolute_controls_are_bounded_monotonic_and_compensate_faint_colors() {
        let colors = [gray(0.2), Oklaba::new(0.4, 0.2, -0.1, 0.6), gray(0.8)];

        for color in colors {
            let adjusted = [-1.0, 0.0, 0.25, 0.5, 0.75, 1.0, 2.0].map(|value| {
                (
                    color.perceptual_alpha(value).alpha,
                    color.perceptual_brightness(value).color.l,
                )
            });

            assert_eq!(adjusted[0], (0.0, 0.0));
            assert_eq!(adjusted[1], (0.0, 0.0));
            assert_eq!(adjusted[5], (1.0, 1.0));
            assert_eq!(adjusted[6], (1.0, 1.0));
            assert!(
                adjusted.windows(2).all(|pair| {
                    pair[0].0 <= pair[1].0 && pair[0].1 <= pair[1].1
                })
            );
        }

        let dark = gray(0.2);
        let colorful = Oklaba::new(0.2, 0.3, 0.4, 1.0);
        let light = gray(0.8);
        let compensated_alpha =
            [dark, colorful, light].map(|color| color.perceptual_alpha(0.5).alpha);
        let compensated_lightness = [dark, colorful, light]
            .map(|color| color.perceptual_brightness(0.5).color.l);

        assert!(compensated_alpha.windows(2).all(|pair| pair[0] > pair[1]));
        assert!(
            compensated_lightness
                .windows(2)
                .all(|pair| pair[0] > pair[1])
        );
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
    fn best_contrast_returns_the_candidate_farthest_from_the_base() {
        assert_eq!(gray(0.2).best_contrast([gray(0.1), WHITE]), WHITE);
        assert_eq!(gray(0.8).best_contrast([BLACK, gray(0.9)]), BLACK);
    }

    #[test]
    fn best_contrast_preserves_the_candidate_type_and_breaks_ties_by_order() {
        let first = Srgba::new(0.8, 0.4, 0.2, 0.3);
        let second = Srgba::new(0.8, 0.4, 0.2, 0.9);

        assert_eq!(BLACK.best_contrast([first, second]), first);
    }

    #[test]
    #[should_panic(expected = "best_contrast requires at least one candidate")]
    fn best_contrast_rejects_an_empty_candidate_array() {
        let _: Oklaba = BLACK.best_contrast([]);
    }

    #[test]
    fn feedback_is_monotonic_with_uniform_displayed_steps_and_neutral_limits() {
        let colors = [
            BLACK,
            gray(0.02),
            gray(0.5),
            gray(0.98),
            WHITE,
            Srgba::new(106.0 / 255.0, 65.0 / 255.0, 1.0, 1.0).into_color(),
            Srgba::new(1.0, 0.0, 0.0, 1.0).into_color(),
            Srgba::new(0.0, 1.0, 1.0, 1.0).into_color(),
            Srgba::new(0.0, 0.0, 1.0, 1.0).into_color(),
        ];

        for color in colors {
            assert_eq!(color.perceptual_feedback(0.0), color);
            let mut previous_lightness = -1.0;

            for amount in [
                -2.0_f32, -1.0, -0.25, -0.08, -0.04, -1e-5, 0.0, 1e-5, 0.04, 0.08,
                0.25, 1.0, 2.0,
            ] {
                let adjusted = color.perceptual_feedback(amount);
                let hsl: Hsla = adjusted.into_color();
                let displayed: Oklaba = hsl.into_color();
                let lightness = displayed.color.l;
                let step = lightness - color.color.l;

                assert!(lightness.is_finite());
                assert!(lightness >= previous_lightness - 1e-6);
                assert!(step.abs() <= amount.abs() + 1e-5);
                if amount > 0.0 {
                    assert!(step >= -1e-6);
                } else if amount < 0.0 {
                    assert!(step <= 1e-6);
                }

                if adjusted.color.l == 0.0 || adjusted.color.l == 1.0 {
                    assert_eq!(adjusted.color.a, 0.0);
                    assert_eq!(adjusted.color.b, 0.0);
                } else {
                    assert_close_within(step, amount, 1e-5);
                }
                if amount <= -1.0 {
                    assert_eq!(adjusted, BLACK);
                } else if amount >= 1.0 {
                    assert_eq!(adjusted, WHITE);
                }
                previous_lightness = lightness;
            }
        }
    }

    #[test]
    fn feedback_preserves_color_and_alpha_across_srgb_and_oklab_inputs() {
        // Sample the sRGB cube, including its saturated edges and neutrals.
        for red in [0.0, 0.25, 0.5, 0.75, 1.0] {
            for green in [0.0, 0.25, 0.5, 0.75, 1.0] {
                for blue in [0.0, 0.25, 0.5, 0.75, 1.0] {
                    for alpha in [0.0, 0.4, 1.0] {
                        let srgb = Srgba::new(red, green, blue, alpha);
                        let original: Oklaba = srgb.into_color();
                        assert_eq!(srgb.perceptual_feedback(0.0), srgb);

                        for amount in [-0.08, -0.04, 0.04, 0.08] {
                            let adjusted = original.perceptual_feedback(amount);
                            let rgb: Srgba = srgb.perceptual_feedback(amount);
                            let roundtrip: Oklaba = rgb.into_color();
                            let raw: LinSrgb =
                                LinSrgb::from_color_unclamped(adjusted.color);

                            for channel in [raw.red, raw.green, raw.blue] {
                                assert!(channel.is_finite());
                                assert!((-1e-6..=1.0 + 1e-6).contains(&channel));
                            }
                            assert_eq!(adjusted.alpha, alpha);
                            assert_eq!(rgb.alpha, alpha);
                            assert_close_within(
                                roundtrip.color.l,
                                adjusted.color.l,
                                1e-5,
                            );
                            assert_close_within(
                                roundtrip.color.a,
                                adjusted.color.a,
                                1e-5,
                            );
                            assert_close_within(
                                roundtrip.color.b,
                                adjusted.color.b,
                                1e-5,
                            );

                            let chroma = original.color.a.hypot(original.color.b);
                            let adjusted_chroma =
                                adjusted.color.a.hypot(adjusted.color.b);
                            assert!(adjusted_chroma <= chroma + 1e-6);
                            if chroma > 1e-4
                                && adjusted.color.l > 1e-4
                                && adjusted.color.l < 1.0 - 1e-4
                            {
                                assert!(adjusted_chroma > 1e-4);
                                assert_close_within(
                                    adjusted.color.a / adjusted_chroma,
                                    original.color.a / chroma,
                                    1e-5,
                                );
                                assert_close_within(
                                    adjusted.color.b / adjusted_chroma,
                                    original.color.b / chroma,
                                    1e-5,
                                );
                            }

                            let mut unchanged_chroma = original;
                            unchanged_chroma.color.l = adjusted.color.l;
                            let rgb: LinSrgb = LinSrgb::from_color_unclamped(
                                unchanged_chroma.color,
                            );
                            if rgb.is_within_bounds()
                                && adjusted.color.l > 0.0
                                && adjusted.color.l < 1.0
                            {
                                assert_eq!(adjusted.color.a, original.color.a);
                                assert_eq!(adjusted.color.b, original.color.b);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn absolute_controls_change_only_their_documented_components() {
        let oklab: Oklaba = Srgba::new(0.4, 0.3, 0.5, 0.2).into_color();
        let adjusted_alpha = oklab.perceptual_alpha(0.5);

        assert_eq!(adjusted_alpha.color, oklab.color);
        let adjusted = oklab.perceptual_brightness(0.5);
        assert_close(adjusted.color.a, oklab.color.a);
        assert_close(adjusted.color.b, oklab.color.b);
        assert_close(adjusted.alpha, oklab.alpha);

        let srgb = Srgba::new(0.2, 0.2, 0.2, 0.4);
        let adjusted_alpha: Srgba = srgb.perceptual_alpha(0.5);
        let adjusted_brightness: Srgba = srgb.perceptual_brightness(0.5);

        assert_eq!(adjusted_alpha.color, srgb.color);
        assert_close(adjusted_brightness.alpha, srgb.alpha);
        assert!(adjusted_brightness.red > srgb.red);
        assert_close(adjusted_brightness.red, adjusted_brightness.green);
        assert_close(adjusted_brightness.green, adjusted_brightness.blue);
    }
}
