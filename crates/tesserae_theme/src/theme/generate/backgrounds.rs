use std::array;

use palette::{FromColor, Mix, Oklab, Oklch};

const BACKGROUND_BASE_ACCENT_PRIMARY_MIX: f32 = 0.02;

const RAMP_INTERVAL_COUNT: f32 = 4.;
const LIGHTNESS_RANGE: f32 = 0.225;
const CHROMA_RANGE: f32 = 0.02;

pub fn generate_backgrounds<const C: usize>(
    background_base: Oklab,
    accent_primary: Oklab,
) -> [Oklab; C] {
    let background =
        background_base.mix(accent_primary, BACKGROUND_BASE_ACCENT_PRIMARY_MIX);

    let accent_hue = Oklch::from_color(accent_primary).hue.into_radians();
    let target_lightness = if background.l < 0.5 { 1.0 } else { 0.0 };

    array::from_fn(|index| {
        if index == 0 {
            return background;
        }

        let progress = index as f32 / RAMP_INTERVAL_COUNT;
        let lightness = background.l
            + (target_lightness - background.l) * progress * LIGHTNESS_RANGE;
        let chroma = progress * CHROMA_RANGE;

        Oklab::new(
            lightness,
            background.a + chroma * accent_hue.cos(),
            background.b + chroma * accent_hue.sin(),
        )
    })
}
