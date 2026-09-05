use palette::{
    Clamp, FromColor, Hsla, IntoColor, IsWithinBounds, LinSrgba, Mix, Oklaba, Srgba,
    WithAlpha, color_difference::EuclideanDistance, convert::FromColorUnclamped,
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

fn feedback_candidate(base: Oklaba, endpoint: Oklaba, progress: f32) -> Oklaba {
    let candidate = base.mix(endpoint, progress);
    let rgb: LinSrgba = LinSrgba::from_color_unclamped(candidate);

    // Clipping keeps feedback continuous near saturated blue.
    rgb.clamp().into_color()
}

fn displayed_feedback_color(color: Oklaba) -> Oklaba {
    // GPUI converts solid fills through HSL before displaying them.
    let hsl: Hsla = color.into_color();
    let rgb: Srgba = hsl.into_color();
    rgb.into_color()
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

    /// Applies feedback measured as distance in OKLab.
    ///
    /// Positive amounts move toward white, negative amounts toward black.
    /// The distance includes changes in lightness and chroma. `amount` must be
    /// finite and is clamped to `[-1.0, 1.0]`. Movement stops at black or white.
    /// Zero returns `self` unchanged.
    ///
    /// Starts from the color GPUI displays. Preserves alpha and reduces chroma.
    /// Clipping to sRGB may shift hue slightly. Equal amounts target equal color
    /// differences for opaque fills. Translucent results depend on the background.
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

        let original: Oklaba = self.into_color();
        let base = displayed_feedback_color(original);
        let endpoint =
            Oklaba::new(if amount > 0.0 { 1.0 } else { 0.0 }, 0.0, 0.0, base.alpha);
        let target = amount.abs().min(1.0);
        let available = base.color.distance(endpoint.color);
        if target >= available {
            return endpoint.into_color();
        }

        let mut progress = target / available;
        let candidate = base.mix(endpoint, progress);
        let rgb: LinSrgba = LinSrgba::from_color_unclamped(candidate);
        let tolerance = (target * 1e-5).max(1e-7);
        let displayed = displayed_feedback_color(candidate);
        if rgb.is_within_bounds()
            && (base.color.distance(displayed.color) - target).abs() <= tolerance
        {
            return candidate.into_color();
        }

        // HSL conversion can change the distance even without clipping,
        // especially near black.
        let (mut lower, mut upper) = (0.0, 1.0);
        let mut best = base;
        let mut best_error = target;
        for _ in 0..24 {
            let candidate = feedback_candidate(base, endpoint, progress);
            let distance = base
                .color
                .distance(displayed_feedback_color(candidate).color);
            let error = (distance - target).abs();
            if error < best_error {
                best = candidate;
                best_error = error;
            }
            if error <= tolerance {
                break;
            }
            if distance < target {
                lower = progress;
            } else {
                upper = progress;
            }
            progress = (lower + upper) * 0.5;
        }
        best.into_color()
    }
}

#[cfg(test)]
mod tests {
    use gpui::Background;
    use palette::{LinSrgb, Oklaba, Srgba, color_difference::EuclideanDistance};

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

    fn feedback_samples() -> impl Iterator<Item = Srgba> {
        let channels = [0.0, 0.25, 0.5, 0.75, 1.0];
        channels
            .into_iter()
            .flat_map(move |red| {
                channels.into_iter().flat_map(move |green| {
                    channels
                        .into_iter()
                        .map(move |blue| Srgba::new(red, green, blue, 1.0))
                })
            })
            .chain([Srgba::new(106.0 / 255.0, 65.0 / 255.0, 1.0, 1.0)])
    }

    fn displayed_color(color: Oklaba) -> Oklaba {
        let rgb: Srgba = Background::from(color).as_solid().unwrap().into_color();
        rgb.into_color()
    }

    #[test]
    fn feedback_changes_smoothly_in_both_directions_until_the_endpoint() {
        let colors = feedback_samples().map(|color| color.into_color()).chain([
            gray(0.0001),
            gray(0.9999),
            // These colors exceed sRGB.
            Oklaba::new(0.55, 0.4, -0.4, 0.4),
            Oklaba::new(0.72, -0.3, 0.3, 0.6),
        ]);
        for color in colors {
            assert_eq!(color.perceptual_feedback(0.0), color);
            let rest = displayed_color(color);
            for direction in [-1.0, 1.0] {
                let endpoint = if direction > 0.0 { WHITE } else { BLACK };
                let available = rest.color.distance(displayed_color(endpoint).color);
                let mut previous = rest;
                let mut previous_amount = 0.0;
                let mut previous_strength = 0.0;
                let amounts = [1e-7, 1e-6, 1e-5, 1e-4]
                    .into_iter()
                    .chain((1..=100).map(|step| step as f32 / 100.0))
                    .chain([2.0]);

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
                    assert!(strength >= previous_strength - 1e-6);
                    assert!(
                        previous.color.distance(displayed.color)
                            <= (amount - previous_amount) * 1.5 + 2e-6,
                        "base {color:?}, amount {}: discontinuous feedback",
                        direction * amount,
                    );
                    if amount >= 1.0 {
                        assert_eq!(adjusted.color, endpoint.color);
                        assert_eq!(adjusted.alpha, color.alpha);
                    }
                    previous = displayed;
                    previous_amount = amount;
                    previous_strength = strength;
                }
            }
        }
    }

    #[test]
    fn feedback_preserves_color_identity_alpha_and_representation() {
        for sample in feedback_samples() {
            for alpha in [0.0, 0.4, 1.0] {
                let srgb = sample.with_alpha(alpha);
                let original: Oklaba = srgb.into_color();
                assert_eq!(srgb.perceptual_feedback(0.0), srgb);

                for amount in [-1.0_f32, -0.08, -0.04, 0.04, 0.08, 1.0] {
                    let adjusted = original.perceptual_feedback(amount);
                    let rgb: Srgba = srgb.perceptual_feedback(amount);
                    let displayed = displayed_color(adjusted);
                    let roundtrip = displayed_color(rgb.into_color());
                    let raw: LinSrgb = LinSrgb::from_color_unclamped(adjusted.color);

                    for channel in [raw.red, raw.green, raw.blue] {
                        assert!(channel.is_finite());
                        // Allow conversion rounding at the sRGB limits.
                        assert!(
                            (-1e-5..=1.0 + 1e-5).contains(&channel),
                            "base {srgb:?}, amount {amount}, raw output {raw:?}",
                        );
                    }
                    assert_eq!(adjusted.alpha, alpha);
                    assert_eq!(rgb.alpha, alpha);
                    assert!(rgb.is_within_bounds());
                    assert!(displayed.color.distance(roundtrip.color) < 1e-5);

                    let chroma = original.color.a.hypot(original.color.b);
                    let adjusted_chroma = displayed.color.a.hypot(displayed.color.b);
                    assert!(adjusted_chroma <= chroma + 1e-5);
                    if chroma < 1e-5 {
                        assert!(adjusted_chroma < 1e-5);
                    } else if amount.abs() <= 0.08
                        && chroma > 0.05
                        && adjusted_chroma > 0.05
                    {
                        let hue_change =
                            (displayed.color.b.atan2(displayed.color.a)
                                - original.color.b.atan2(original.color.a))
                            .to_degrees();
                        let hue_change =
                            (hue_change + 180.0).rem_euclid(360.0) - 180.0;
                        assert!(
                            hue_change.abs() < 2.0,
                            "base {srgb:?}, hue change {hue_change}"
                        );
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
