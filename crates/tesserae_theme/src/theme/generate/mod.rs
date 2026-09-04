use build_setters_macro::BuildSetters;
use gpui::{DefiniteLength, Rems, relative, rems};
use palette::{IntoColor, Mix, Oklab, Oklaba};

use crate::{Theme, ThemeSetKind, color_from_hex};

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

const SECONDARY_ACCENT_MIX: f32 = 0.15;
const SECONDARY_ACCENT_TINT_MIX: f32 = 0.08;

fn mix_chroma(color: Oklaba, other: Oklaba, factor: f32) -> Oklaba {
    color.mix(
        Oklaba::new(color.color.l, other.color.a, other.color.b, color.alpha),
        factor,
    )
}

#[derive(BuildSetters, Clone)]
pub struct ThemeConfig {
    pub base_fg: Oklaba,
    pub base_bg: Oklaba,
    pub base_caution: Oklaba,
    pub base_destruct: Oklaba,

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

    pub size_3xl: Rems,
    pub size_2xl: Rems,
    pub size_xl: Rems,
    pub size_lg: Rems,
    pub size_md: Rems,
    pub size_sm: Rems,
    pub size_xs: Rems,
    pub size_2xs: Rems,
    pub size_3xs: Rems,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            base_fg: color_from_hex("E5E5E5FF").unwrap(),
            base_bg: color_from_hex("020617FF").unwrap(),
            base_caution: color_from_hex("EAB308FF").unwrap(),
            base_destruct: color_from_hex("B91C1CFF").unwrap(),

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

            size_3xl: rems(2.25),
            size_2xl: rems(2.),
            size_xl: rems(1.75),
            size_lg: rems(1.5),
            size_md: rems(1.25),
            size_sm: rems(1.),
            size_xs: rems(0.875),
            size_2xs: rems(0.75),
            size_3xs: rems(0.625),
        }
    }
}

impl AsRef<ThemeConfig> for ThemeConfig {
    fn as_ref(&self) -> &ThemeConfig {
        self
    }
}

pub fn generate_theme(config: &ThemeConfig, kind: ThemeSetKind) -> Theme {
    let (base_bg, base_fg, base_fg_inverse) = match kind {
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

    let base_fg_tint: Oklab = config.base_fg.color.into_color();
    let base_bg_tint: Oklab = config.base_bg.color.into_color();

    let backgrounds =
        generate_backgrounds::<BACKGROUND_SHADES>(base_bg, base_bg_tint);
    let foregrounds =
        generate_foregrounds::<FOREGROUND_SHADES>(base_fg, base_fg_tint);
    let foregrounds_inverse =
        generate_foregrounds::<FOREGROUND_SHADES>(base_fg_inverse, base_fg_tint);

    let accent_secondary = mix_chroma(
        base_bg.mix(base_fg, SECONDARY_ACCENT_MIX).into_color(),
        config.base_fg,
        SECONDARY_ACCENT_TINT_MIX,
    );

    Theme {
        bg_primary: backgrounds[0].into_color(),
        bg_secondary: backgrounds[1].into_color(),
        bg_tertiary: backgrounds[2].into_color(),
        bg_quaternary: backgrounds[3].into_color(),
        bg_quinary: backgrounds[4].into_color(),
        bg_senary: backgrounds[5].into_color(),

        fg_primary: foregrounds[0].into_color(),
        fg_secondary: foregrounds[1].into_color(),
        fg_tertiary: foregrounds[2].into_color(),

        fg_inverse_primary: foregrounds_inverse[0].into_color(),
        fg_inverse_secondary: foregrounds_inverse[1].into_color(),
        fg_inverse_tertiary: foregrounds_inverse[2].into_color(),

        accent_primary: config.base_fg,
        accent_secondary,
        accent_caution: config.base_caution,
        accent_destruct: config.base_destruct,

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

        size_3xl: config.size_3xl,
        size_2xl: config.size_2xl,
        size_xl: config.size_xl,
        size_lg: config.size_lg,
        size_md: config.size_md,
        size_sm: config.size_sm,
        size_xs: config.size_xs,
        size_2xs: config.size_2xs,
        size_3xs: config.size_3xs,
    }
}
