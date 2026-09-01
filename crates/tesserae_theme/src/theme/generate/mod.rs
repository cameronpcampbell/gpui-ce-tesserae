use std::str::FromStr;

use build_setters_macro::BuildSetters;
use gpui::{DefiniteLength, Pixels, Rems, Rgba, px, relative, rems};
use palette::{Clamp, IntoColor, Mix, Oklab, Oklaba, convert::FromColorUnclamped};

use crate::{Theme, ThemeSetKind};

mod backgrounds;
use backgrounds::generate_backgrounds;
mod foregrounds;
use foregrounds::generate_foregrounds;

const BACKGROUND_DARK_BASE: Oklab = Oklab::new(0.18220370283599863, 0.0, 0.0);
const BACKGROUND_LIGHT_BASE: Oklab = Oklab::new(
    0.9636238258905064,
    0.011193528098987493,
    -0.0076410071675384605,
);

const FOREGROUND_DARK_BASE: Oklab = Oklab::new(
    0.9841518695012076,
    -0.0012862701774188356,
    -0.0031610164420114994,
);
const FOREGROUND_LIGHT_BASE: Oklab = Oklab::new(
    0.1407653343343911,
    0.0011958779960162258,
    -0.004219718844658132,
);

const BACKGROUND_SHADES: usize = 6;
const FOREGROUND_SHADES: usize = 3;

const SECONDARY_ACCENT_MIX: f32 = 0.17;

#[derive(BuildSetters, Clone)]
pub struct ThemeConfig {
    pub accent_primary: Oklaba,
    pub accent_caution: Oklaba,
    pub accent_destruct: Oklaba,

    pub line_height: DefiniteLength,

    pub text_size_xs: Rems,
    pub text_size_sm: Rems,
    pub text_size_base: Rems,
    pub text_size_lg: Rems,
    pub text_size_xl: Rems,
    pub text_size_2xl: Rems,
    pub text_size_3xl: Rems,
    pub text_size_4xl: Rems,
    pub text_size_5xl: Rems,
    pub text_size_6xl: Rems,
    pub text_size_7xl: Rems,
    pub text_size_8xl: Rems,
    pub text_size_9xl: Rems,

    pub radii_xs: Rems,
    pub radii_sm: Rems,
    pub radii_md: Rems,
    pub radii_lg: Rems,
    pub radii_xl: Rems,
    pub radii_2xl: Rems,
    pub radii_3xl: Rems,
    pub radii_4xl: Rems,

    pub size_xl: Pixels,
    pub size_lg: Pixels,
    pub size_md: Pixels,
    pub size_sm: Pixels,
    pub size_xs: Pixels,
}

fn color_from_hex<T: FromColorUnclamped<Rgba> + Clamp>(
    hex_code: &str,
) -> Result<T, <Rgba as FromStr>::Err> {
    (Rgba::from_hex(hex_code) as Result<Rgba, <Rgba as FromStr>::Err>)
        .map(|color| color.into_color())
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            accent_primary: color_from_hex("3151EFFF").unwrap(),
            accent_caution: color_from_hex("3151EFFF").unwrap(),
            accent_destruct: color_from_hex("3151EFFF").unwrap(),

            line_height: relative(1.618).into(),

            text_size_xs: rems(0.75),
            text_size_sm: rems(0.875),
            text_size_base: rems(1.0),
            text_size_lg: rems(1.125),
            text_size_xl: rems(1.25),
            text_size_2xl: rems(1.5),
            text_size_3xl: rems(1.875),
            text_size_4xl: rems(2.25),
            text_size_5xl: rems(3.0),
            text_size_6xl: rems(3.75),
            text_size_7xl: rems(4.5),
            text_size_8xl: rems(6.0),
            text_size_9xl: rems(8.0),

            radii_xs: rems(0.125),
            radii_sm: rems(0.375),
            radii_md: rems(0.5),
            radii_lg: rems(0.625),
            radii_xl: rems(0.875),
            radii_2xl: rems(1.125),
            radii_3xl: rems(1.375),
            radii_4xl: rems(1.625),

            size_xl: px(36.),
            size_lg: px(32.),
            size_md: px(28.),
            size_sm: px(24.),
            size_xs: px(20.),
        }
    }
}

impl AsRef<ThemeConfig> for ThemeConfig {
    fn as_ref(&self) -> &ThemeConfig {
        self
    }
}

pub fn generate_theme(config: &ThemeConfig, kind: ThemeSetKind) -> Theme {
    let (background_base, foreground_base, foreground_inverse_base) = match kind {
        ThemeSetKind::Light => (
            BACKGROUND_LIGHT_BASE,
            FOREGROUND_LIGHT_BASE,
            FOREGROUND_DARK_BASE,
        ),
        ThemeSetKind::Dark => (
            BACKGROUND_DARK_BASE,
            FOREGROUND_DARK_BASE,
            FOREGROUND_LIGHT_BASE,
        ),
    };

    let accent_primary: Oklab = config.accent_primary.color.into_color();

    let backgrounds =
        generate_backgrounds::<BACKGROUND_SHADES>(background_base, accent_primary);
    let foregrounds =
        generate_foregrounds::<FOREGROUND_SHADES>(foreground_base, accent_primary);
    let foregrounds_inverse = generate_foregrounds::<FOREGROUND_SHADES>(
        foreground_inverse_base,
        accent_primary,
    );

    let foreground_primary = foregrounds[0];
    let accent_secondary =
        foreground_primary.mix(accent_primary, SECONDARY_ACCENT_MIX);

    Theme {
        bg_primary: backgrounds[0].into_color(),
        bg_secondary: backgrounds[1].into_color(),
        bg_tertiary: backgrounds[2].into_color(),
        bg_quaternary: backgrounds[3].into_color(),
        bg_quinary: backgrounds[4].into_color(),
        bg_senary: backgrounds[5].into_color(),

        fg_primary: foreground_primary.into_color(),
        fg_secondary: foregrounds[1].into_color(),
        fg_tertiary: foregrounds[2].into_color(),

        fg_inverse_primary: foregrounds_inverse[0].into_color(),
        fg_inverse_secondary: foregrounds_inverse[1].into_color(),
        fg_inverse_tertiary: foregrounds_inverse[2].into_color(),

        accent_primary: config.accent_primary.into_color(),
        accent_secondary: accent_secondary.into_color(),
        accent_caution: config.accent_caution,
        accent_destruct: config.accent_destruct,

        line_height: config.line_height,

        text_size_xs: config.text_size_xs,
        text_size_sm: config.text_size_sm,
        text_size_base: config.text_size_base,
        text_size_lg: config.text_size_lg,
        text_size_xl: config.text_size_xl,
        text_size_2xl: config.text_size_2xl,
        text_size_3xl: config.text_size_3xl,
        text_size_4xl: config.text_size_4xl,
        text_size_5xl: config.text_size_5xl,
        text_size_6xl: config.text_size_6xl,
        text_size_7xl: config.text_size_7xl,
        text_size_8xl: config.text_size_8xl,
        text_size_9xl: config.text_size_9xl,

        radii_xs: config.radii_xs,
        radii_sm: config.radii_sm,
        radii_md: config.radii_md,
        radii_lg: config.radii_lg,
        radii_xl: config.radii_xl,
        radii_2xl: config.radii_2xl,
        radii_3xl: config.radii_3xl,
        radii_4xl: config.radii_4xl,

        size_xl: config.size_xl,
        size_lg: config.size_lg,
        size_md: config.size_md,
        size_sm: config.size_sm,
        size_xs: config.size_xs,
    }
}
