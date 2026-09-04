use palette::{FromColor, IntoColor, Oklaba, WithAlpha};

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

fn luminance_from_whittle_contrast(contrast: f32, surround: f32) -> f32 {
    if contrast >= 0.0 {
        surround + contrast * (surround + FLARE_LUMINANCE)
    } else {
        (surround + contrast * FLARE_LUMINANCE) / (1.0 - contrast)
    }
}

fn brightness(contrast: f32) -> f32 {
    let gain = if contrast < 0.0 { DECREMENT_GAIN } else { 1.0 };

    gain * contrast.signum() * (1.0 + CONTRAST_GAIN * contrast.abs()).ln()
}

fn contrast_from_brightness(brightness: f32) -> f32 {
    let gain = if brightness < 0.0 {
        DECREMENT_GAIN
    } else {
        1.0
    };

    brightness.signum() * ((brightness.abs() / gain).exp() - 1.0) / CONTRAST_GAIN
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

fn nudge_lightness(lightness: f32, surround_lightness: f32, amount: f32) -> f32 {
    let surround = luminance(surround_lightness);
    let brightness = brightness(whittle_contrast(luminance(lightness), surround))
        + amount.clamp(-1.0, 1.0) * brightness_range();
    let contrast = contrast_from_brightness(brightness);

    luminance_from_whittle_contrast(contrast, surround)
        .clamp(0.0, 1.0)
        .cbrt()
}

/// Perceptual controls that adjust colors in OKLab and return the input type.
pub trait PerceptualColor: Sized {
    /// Applies a perceptual alpha to the color using its lightness and chroma.
    fn perceptual_alpha(self, desired_alpha: f32) -> Self;

    /// Applies a perceptual brightness to the color using its lightness and chroma.
    fn perceptual_brightness(self, intensity: f32) -> Self;

    /// Applies perceptual feedback to the color using its surround.
    fn perceptual_feedback(
        self,
        amount: f32,
        surround: impl IntoColor<Oklaba>,
    ) -> Self;
}

impl<C> PerceptualColor for C
where
    C: Clone + IntoColor<Oklaba> + FromColor<Oklaba> + WithAlpha<f32, WithAlpha = C>,
{
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

    fn perceptual_feedback(
        self,
        amount: f32,
        surround: impl IntoColor<Oklaba>,
    ) -> Self {
        let mut perceptual_color: Oklaba = self.into_color();
        let surround: Oklaba = surround.into_color();

        perceptual_color.color.l =
            nudge_lightness(perceptual_color.color.l, surround.color.l, amount);
        perceptual_color.into_color()
    }
}

#[cfg(test)]
mod tests {
    use palette::{Oklaba, Srgba};

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
    fn feedback_adds_the_requested_contrast_until_lightness_reaches_a_bound() {
        let colors = [
            gray(0.02),
            gray(0.5),
            gray(0.98),
            Oklaba::new(0.5, 0.15, 0.1, 0.4),
            Oklaba::new(0.7, -0.1, 0.12, 0.4),
        ];
        let surrounds = [gray(0.0), gray(0.12), gray(0.5), gray(0.95), WHITE];

        for color in colors {
            for surround in surrounds {
                let original = perceptual_contrast(color, surround);

                for amount in [-0.12_f32, -0.03, 0.03, 0.12] {
                    let adjusted = color.perceptual_feedback(amount, surround);
                    let step = perceptual_contrast(adjusted, surround) - original;
                    let reached_bound =
                        adjusted.color.l == 0.0 || adjusted.color.l == 1.0;

                    if reached_bound {
                        assert!(step.abs() <= amount.abs() + 1e-5);
                        assert_eq!(step.signum(), amount.signum());
                    } else {
                        assert_close_within(step, amount, 1e-5);
                    }
                }
            }
        }

        for surround in [gray(0.0), gray(0.5), gray(1.0)] {
            for (amount, expected) in
                [(-2.0, 0.0), (-1.0, 0.0), (0.0, 0.6), (1.0, 1.0), (2.0, 1.0)]
            {
                assert_close(
                    gray(0.6).perceptual_feedback(amount, surround).color.l,
                    expected,
                );
            }
        }
    }

    #[test]
    fn feedback_needs_a_larger_lightness_change_farther_from_the_surround() {
        let surround = gray(0.3);
        let near = gray(0.3).perceptual_feedback(0.04, surround).color.l - 0.3;
        let far = gray(0.75).perceptual_feedback(0.04, surround).color.l - 0.75;

        assert!(near > 0.0);
        assert!(far > near * 1.5, "far {far} should exceed near {near}");
    }

    #[test]
    fn adjustments_change_only_their_documented_components() {
        let oklab: Oklaba = Srgba::new(0.4, 0.3, 0.5, 0.2).into_color();
        let adjusted_alpha = oklab.perceptual_alpha(0.5);

        assert_eq!(adjusted_alpha.color, oklab.color);
        for adjusted in [
            oklab.perceptual_brightness(0.5),
            oklab.perceptual_feedback(-0.03, BLACK),
        ] {
            assert_close(adjusted.color.a, oklab.color.a);
            assert_close(adjusted.color.b, oklab.color.b);
            assert_close(adjusted.alpha, oklab.alpha);
        }

        let srgb = Srgba::new(0.2, 0.2, 0.2, 0.4);
        let adjusted_alpha: Srgba = srgb.perceptual_alpha(0.5);
        let adjusted_brightness: Srgba = srgb.perceptual_brightness(0.5);
        let adjusted_feedback: Srgba = srgb.perceptual_feedback(0.1, BLACK);

        assert_eq!(adjusted_alpha.color, srgb.color);
        for adjusted in [adjusted_brightness, adjusted_feedback] {
            assert_close(adjusted.alpha, srgb.alpha);
            assert!(adjusted.red > srgb.red);
            assert_close(adjusted.red, adjusted.green);
            assert_close(adjusted.green, adjusted.blue);
        }
    }
}
