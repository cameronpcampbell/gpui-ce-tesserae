use std::array;

use palette::{FromColor, Mix, Oklab, Oklch};

const FOREGROUND_ACCENT_PRIMARY_MIX: f32 = 0.05;

const RAMP_INTERVAL_COUNT: f32 = 4.0;
const LIGHT_FOREGROUND_LIGHTNESS_RANGE: f32 = 1.0;
const DARK_FOREGROUND_LIGHTNESS_RANGE: f32 = 0.62;
const CHROMA_RANGE: f32 = 0.017;

pub fn generate_foregrounds<const C: usize>(
    foreground_base: Oklab,
    accent_primary: Oklab,
) -> [Oklab; C] {
    let (foreground_base_chroma, foreground_base_hue) = {
        let foreground_base = Oklch::from_color(foreground_base);
        (foreground_base.chroma, foreground_base.hue)
    };

    let is_dark = foreground_base.l < 0.5;
    let target_lightness = if is_dark { 1.0 } else { 0.0 };
    let lightness_range = if is_dark {
        LIGHT_FOREGROUND_LIGHTNESS_RANGE
    } else {
        DARK_FOREGROUND_LIGHTNESS_RANGE
    };

    array::from_fn(|index| {
        let progress = index as f32 / RAMP_INTERVAL_COUNT;

        let lightness = foreground_base.l
            + (target_lightness - foreground_base.l) * progress * lightness_range;
        let chroma = foreground_base_chroma + progress * CHROMA_RANGE;

        let foreground =
            Oklab::from_color(Oklch::new(lightness, chroma, foreground_base_hue));

        foreground.mix(accent_primary, FOREGROUND_ACCENT_PRIMARY_MIX)
    })
}
