use std::{fmt, str::FromStr};

use gpui::{App, DefiniteLength, Relative, Rems, Rgba};
use palette::{
    Clamp, IntoColor, Oklaba, Srgba, color_difference::Wcag21RelativeContrast,
    convert::FromColorUnclamped,
};
use tesserae_utils::PerceptualColor;

use crate::{ThemeSetKind, ThemeSetKindState, ThemeSetState};

mod generate;
pub use generate::ThemeConfig;
use generate::generate_theme;

pub fn color_from_hex<Color: FromColorUnclamped<Rgba> + Clamp>(
    hex_code: &str,
) -> Result<Color, <Rgba as FromStr>::Err> {
    (Rgba::from_hex(hex_code) as Result<Rgba, <Rgba as FromStr>::Err>)
        .map(|color| color.into_color())
}

#[derive(Clone, derive_more::Debug)]
pub struct Theme {
    #[debug("{:?}", HexColor(*bg_primary))]
    pub bg_primary: Oklaba,
    #[debug("{:?}", HexColor(*bg_secondary))]
    pub bg_secondary: Oklaba,
    #[debug("{:?}", HexColor(*bg_tertiary))]
    pub bg_tertiary: Oklaba,
    #[debug("{:?}", HexColor(*bg_quaternary))]
    pub bg_quaternary: Oklaba,
    #[debug("{:?}", HexColor(*bg_quinary))]
    pub bg_quinary: Oklaba,
    #[debug("{:?}", HexColor(*bg_senary))]
    pub bg_senary: Oklaba,

    #[debug("{:?}", HexColor(*fg_primary))]
    pub fg_primary: Oklaba,
    #[debug("{:?}", HexColor(*fg_secondary))]
    pub fg_secondary: Oklaba,
    #[debug("{:?}", HexColor(*fg_tertiary))]
    pub fg_tertiary: Oklaba,

    #[debug("{:?}", HexColor(*fg_inverse_primary))]
    pub fg_inverse_primary: Oklaba,
    #[debug("{:?}", HexColor(*fg_inverse_secondary))]
    pub fg_inverse_secondary: Oklaba,
    #[debug("{:?}", HexColor(*fg_inverse_tertiary))]
    pub fg_inverse_tertiary: Oklaba,

    #[debug("{:?}", HexColor(*accent_primary))]
    pub accent_primary: Oklaba,
    #[debug("{:?}", HexColor(*accent_secondary))]
    pub accent_secondary: Oklaba,
    #[debug("{:?}", HexColor(*accent_caution))]
    pub accent_caution: Oklaba,
    #[debug("{:?}", HexColor(*accent_destruct))]
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

    pub size_xs: Rems,
    pub size_sm: Rems,
    pub size_md: Rems,
    pub size_lg: Rems,
    pub size_xl: Rems,
    pub size_2xl: Rems,
    pub size_3xl: Rems,
    pub size_4xl: Rems,
    pub size_5xl: Rems,
    pub size_6xl: Rems,

    pub thumb_ratio: Relative,

    pub radii_xs: Rems,
    pub radii_sm: Rems,
    pub radii_md: Rems,
    pub radii_lg: Rems,
    pub radii_xl: Rems,
    pub radii_2xl: Rems,
    pub radii_3xl: Rems,
    pub radii_4xl: Rems,

    pub padding_xs: Rems,
    pub padding_sm: Rems,
    pub padding_md: Rems,
    pub padding_lg: Rems,
    pub padding_xl: Rems,
    pub padding_2xl: Rems,
    pub padding_3xl: Rems,
    pub padding_4xl: Rems,
    pub padding_5xl: Rems,
}

#[allow(
    dead_code,
    reason = "constructed by Theme's derived Debug implementation"
)]
struct HexColor(Oklaba);

#[inline(always)]
fn contrast_ratio(
    foreground: impl IntoColor<Rgba>,
    background: impl IntoColor<Rgba>,
) -> f32 {
    foreground
        .into_color()
        .color
        .relative_contrast(background.into_color().color)
}

impl fmt::Debug for HexColor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rgba: Rgba = self.0.into_color();
        let rgba: Srgba<u8> = rgba.into_format();

        write!(formatter, "#{rgba:x}")
    }
}

impl Theme {
    const HOVER_FEEDBACK: f32 = 0.025;
    const ACTIVE_FEEDBACK: f32 = 0.05;

    pub fn read_global(cx: &App) -> &Theme {
        let theme_set_kind = *cx.global::<ThemeSetKindState>().0.read(cx);
        let theme_set = cx.global::<ThemeSetState>().0.read(cx);

        theme_set.get_theme(theme_set_kind)
    }

    pub fn generate(config: &ThemeConfig, kind: ThemeSetKind) -> Self {
        generate_theme(config, kind)
    }

    pub fn background(&self, kind: ThemeBgKind) -> Oklaba {
        match kind {
            ThemeBgKind::Primary => self.bg_primary,
            ThemeBgKind::Secondary => self.bg_secondary,
            ThemeBgKind::Tertiary => self.bg_tertiary,
            ThemeBgKind::Quaternary => self.bg_quaternary,
            ThemeBgKind::Quinary => self.bg_quinary,
            ThemeBgKind::Senary => self.bg_senary,
        }
    }

    pub fn background_border(&self, kind: ThemeBgBorderKind) -> Oklaba {
        match kind {
            ThemeBgBorderKind::Primary => self.bg_secondary,
            ThemeBgBorderKind::Secondary => self.bg_tertiary,
            ThemeBgBorderKind::Tertiary => self.bg_quaternary,
            ThemeBgBorderKind::Quaternary => self.bg_quinary,
            ThemeBgBorderKind::Quinary => self.bg_senary,
        }
    }

    pub fn foreground(&self, kind: ThemeFgKind) -> Oklaba {
        match kind {
            ThemeFgKind::Primary => self.fg_primary,
            ThemeFgKind::Secondary => self.fg_secondary,
            ThemeFgKind::Tertiary => self.fg_tertiary,
        }
    }

    pub fn foreground_inverse(&self, kind: ThemeFgKind) -> Oklaba {
        match kind {
            ThemeFgKind::Primary => self.fg_inverse_primary,
            ThemeFgKind::Secondary => self.fg_inverse_secondary,
            ThemeFgKind::Tertiary => self.fg_inverse_tertiary,
        }
    }

    pub fn foreground_for_background(
        &self,
        kind: ThemeFgKind,
        background: impl IntoColor<Oklaba>,
    ) -> Oklaba {
        let background = background.into_color();

        let (foreground, foreground_inverse) = match kind {
            ThemeFgKind::Primary => (self.fg_primary, self.fg_inverse_primary),
            ThemeFgKind::Secondary => (self.fg_secondary, self.fg_inverse_secondary),
            ThemeFgKind::Tertiary => (self.fg_tertiary, self.fg_inverse_tertiary),
        };

        if contrast_ratio(foreground, background)
            >= contrast_ratio(foreground_inverse, background)
        {
            foreground
        } else {
            foreground_inverse
        }
    }

    pub fn hover_feedback(&self, color: impl IntoColor<Oklaba>) -> Oklaba {
        let color = color.into_color();
        let amount = self.feedback_direction(color) * Self::HOVER_FEEDBACK;

        color.perceptual_feedback(amount)
    }

    pub fn active_feedback(&self, color: impl IntoColor<Oklaba>) -> Oklaba {
        let color = color.into_color();
        let amount = self.feedback_direction(color) * Self::ACTIVE_FEEDBACK;

        color.perceptual_feedback(amount)
    }

    fn feedback_direction(&self, color: Oklaba) -> f32 {
        let foreground_contrast = contrast_ratio(color, self.fg_primary);
        let background_contrast = contrast_ratio(color, self.bg_primary);
        let target = if foreground_contrast >= background_contrast {
            self.fg_primary
        } else {
            self.bg_primary
        };

        let lightness_difference = target.color.l - color.color.l;

        if lightness_difference > f32::EPSILON {
            1.0
        } else if lightness_difference < -f32::EPSILON {
            -1.0
        } else if color.color.l <= 0.5 {
            1.0
        } else {
            -1.0
        }
    }

    pub fn accent(&self, kind: ThemeAccentKind) -> Oklaba {
        match kind {
            ThemeAccentKind::Primary => self.accent_primary,
            ThemeAccentKind::Secondary => self.accent_secondary,
            ThemeAccentKind::Caution => self.accent_caution,
            ThemeAccentKind::Destruct => self.accent_destruct,
        }
    }

    pub fn text_size(&self, kind: ThemeTextSizeKind) -> Rems {
        match kind {
            ThemeTextSizeKind::ExtraSmall => self.text_size_xs,
            ThemeTextSizeKind::Small => self.text_size_sm,
            ThemeTextSizeKind::Base => self.text_size_base,
            ThemeTextSizeKind::Large => self.text_size_lg,
            ThemeTextSizeKind::ExtraLarge => self.text_size_xl,
            ThemeTextSizeKind::X2l => self.text_size_2xl,
            ThemeTextSizeKind::X3l => self.text_size_3xl,
            ThemeTextSizeKind::X4l => self.text_size_4xl,
            ThemeTextSizeKind::X5l => self.text_size_5xl,
            ThemeTextSizeKind::X6l => self.text_size_6xl,
            ThemeTextSizeKind::X7l => self.text_size_7xl,
            ThemeTextSizeKind::X8l => self.text_size_8xl,
            ThemeTextSizeKind::X9l => self.text_size_9xl,
        }
    }

    pub fn radii(&self, kind: ThemeRadiiKind) -> Rems {
        match kind {
            ThemeRadiiKind::ExtraSmall => self.radii_xs,
            ThemeRadiiKind::Small => self.radii_sm,
            ThemeRadiiKind::Medium => self.radii_md,
            ThemeRadiiKind::Large => self.radii_lg,
            ThemeRadiiKind::ExtraLarge => self.radii_xl,
            ThemeRadiiKind::X2l => self.radii_2xl,
            ThemeRadiiKind::X3l => self.radii_3xl,
            ThemeRadiiKind::X4l => self.radii_4xl,
        }
    }

    pub fn size(&self, kind: ThemeSizeKind) -> Rems {
        match kind {
            ThemeSizeKind::ExtraSmall => self.size_xs,
            ThemeSizeKind::Small => self.size_sm,
            ThemeSizeKind::Medium => self.size_md,
            ThemeSizeKind::Large => self.size_lg,
            ThemeSizeKind::ExtraLarge => self.size_xl,
            ThemeSizeKind::X2l => self.size_2xl,
            ThemeSizeKind::X3l => self.size_3xl,
            ThemeSizeKind::X4l => self.size_4xl,
            ThemeSizeKind::X5l => self.size_5xl,
            ThemeSizeKind::X6l => self.size_6xl,
        }
    }

    pub fn padding(&self, kind: ThemePaddingKind) -> Rems {
        match kind {
            ThemePaddingKind::ExtraSmall => self.padding_xs,
            ThemePaddingKind::Small => self.padding_sm,
            ThemePaddingKind::Medium => self.padding_md,
            ThemePaddingKind::Large => self.padding_lg,
            ThemePaddingKind::ExtraLarge => self.padding_xl,
            ThemePaddingKind::X2l => self.padding_2xl,
            ThemePaddingKind::X3l => self.padding_3xl,
            ThemePaddingKind::X4l => self.padding_4xl,
            ThemePaddingKind::X5l => self.padding_5xl,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ThemeBgKind {
    Primary,
    Secondary,
    Tertiary,
    Quaternary,
    Quinary,
    Senary,
}

#[derive(Clone, Copy)]
pub enum ThemeBgBorderKind {
    Primary,
    Secondary,
    Tertiary,
    Quaternary,
    Quinary,
}

#[derive(Clone, Copy)]
pub enum ThemeFgKind {
    Primary,
    Secondary,
    Tertiary,
}

#[derive(Clone, Copy)]
pub enum ThemeAccentKind {
    Primary,
    Secondary,
    Caution,
    Destruct,
}

#[derive(Clone, Copy)]
pub enum ThemeTextSizeKind {
    ExtraSmall,
    Small,
    Base,
    Large,
    ExtraLarge,
    X2l,
    X3l,
    X4l,
    X5l,
    X6l,
    X7l,
    X8l,
    X9l,
}

#[derive(Clone, Copy)]
pub enum ThemeRadiiKind {
    ExtraSmall,
    Small,
    Medium,
    Large,
    ExtraLarge,
    X2l,
    X3l,
    X4l,
}

#[derive(Clone, Copy)]
pub enum ThemeSizeKind {
    ExtraSmall,
    Small,
    Medium,
    Large,
    ExtraLarge,
    X2l,
    X3l,
    X4l,
    X5l,
    X6l,
}

#[derive(Clone, Copy)]
pub enum ThemePaddingKind {
    ExtraSmall,
    Small,
    Medium,
    Large,
    ExtraLarge,
    X2l,
    X3l,
    X4l,
    X5l,
}
