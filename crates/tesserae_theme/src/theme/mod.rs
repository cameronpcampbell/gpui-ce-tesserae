use std::fmt;

use gpui::{App, DefiniteLength, Pixels, Rems, Rgba};
use palette::{
    IntoColor, Mix, Oklaba, Srgba, color_difference::Wcag21RelativeContrast,
};

use crate::{ThemeSetKind, ThemeSetKindState, ThemeSetState};

mod generate;
pub use generate::ThemeConfig;
use generate::generate_theme;

#[derive(Clone)]
pub struct Theme {
    pub bg_primary: Oklaba,
    pub bg_secondary: Oklaba,
    pub bg_tertiary: Oklaba,
    pub bg_quaternary: Oklaba,
    pub bg_quinary: Oklaba,
    pub bg_senary: Oklaba,

    pub fg_primary: Oklaba,
    pub fg_secondary: Oklaba,
    pub fg_tertiary: Oklaba,

    pub fg_inverse_primary: Oklaba,
    pub fg_inverse_secondary: Oklaba,
    pub fg_inverse_tertiary: Oklaba,

    pub accent_primary: Oklaba,
    pub accent_secondary: Oklaba,
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

struct HexColor(Oklaba);

#[inline(always)]
fn contrast_ratio(fg: impl IntoColor<Rgba>, bg: impl IntoColor<Rgba>) -> f32 {
    fg.into_color()
        .color
        .relative_contrast(bg.into_color().color)
}

impl fmt::Debug for HexColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rgba: Rgba = self.0.into_color();
        let rgba: Srgba<u8> = rgba.into_format();

        write!(f, "#{rgba:x}")
    }
}

impl fmt::Debug for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Theme")
            .field("bg_primary", &HexColor(self.bg_primary))
            .field("bg_secondary", &HexColor(self.bg_secondary))
            .field("bg_tertiary", &HexColor(self.bg_tertiary))
            .field("bg_quaternary", &HexColor(self.bg_quaternary))
            .field("bg_quinary", &HexColor(self.bg_quinary))
            .field("bg_senary", &HexColor(self.bg_senary))
            .field("fg_primary", &HexColor(self.fg_primary))
            .field("fg_secondary", &HexColor(self.fg_secondary))
            .field("fg_tertiary", &HexColor(self.fg_tertiary))
            .field("fg_inverse_primary", &HexColor(self.fg_inverse_primary))
            .field("fg_inverse_secondary", &HexColor(self.fg_inverse_secondary))
            .field("fg_inverse_tertiary", &HexColor(self.fg_inverse_tertiary))
            .field("accent_primary", &HexColor(self.accent_primary))
            .field("accent_secondary", &HexColor(self.accent_secondary))
            .field("accent_caution", &HexColor(self.accent_caution))
            .field("accent_destruct", &HexColor(self.accent_destruct))
            .field("line_height", &self.line_height)
            .field("text_size_xs", &self.text_size_xs)
            .field("text_size_sm", &self.text_size_sm)
            .field("text_size_base", &self.text_size_base)
            .field("text_size_lg", &self.text_size_lg)
            .field("text_size_xl", &self.text_size_xl)
            .field("text_size_2xl", &self.text_size_2xl)
            .field("text_size_3xl", &self.text_size_3xl)
            .field("text_size_4xl", &self.text_size_4xl)
            .field("text_size_5xl", &self.text_size_5xl)
            .field("text_size_6xl", &self.text_size_6xl)
            .field("text_size_7xl", &self.text_size_7xl)
            .field("text_size_8xl", &self.text_size_8xl)
            .field("text_size_9xl", &self.text_size_9xl)
            .field("radii_xs", &self.radii_xs)
            .field("radii_sm", &self.radii_sm)
            .field("radii_md", &self.radii_md)
            .field("radii_lg", &self.radii_lg)
            .field("radii_xl", &self.radii_xl)
            .field("radii_2xl", &self.radii_2xl)
            .field("radii_3xl", &self.radii_3xl)
            .field("radii_4xl", &self.radii_4xl)
            .field("size_xl", &self.size_xl)
            .field("size_lg", &self.size_lg)
            .field("size_md", &self.size_md)
            .field("size_sm", &self.size_sm)
            .field("size_xs", &self.size_xs)
            .finish()
    }
}

impl Theme {
    const HOVER_COLOR_MIX: f32 = 0.03;
    const ACTIVE_COLOR_MIX: f32 = 0.05;

    pub fn read_global(cx: &App) -> &Theme {
        let theme_set_kind = *cx.global::<ThemeSetKindState>().0.read(cx);
        let theme_set = cx.global::<ThemeSetState>().0.read(cx);

        theme_set.get_theme(theme_set_kind)
    }

    pub fn generate(config: &ThemeConfig, kind: ThemeSetKind) -> Self {
        generate_theme(config, kind)
    }

    pub fn bg(&self, kind: ThemeBgKind) -> Oklaba {
        match kind {
            ThemeBgKind::Primary => self.bg_primary,
            ThemeBgKind::Secondary => self.bg_secondary,
            ThemeBgKind::Tertiary => self.bg_tertiary,
            ThemeBgKind::Quaternary => self.bg_quaternary,
            ThemeBgKind::Quinary => self.bg_quinary,
            ThemeBgKind::Senary => self.bg_senary,
        }
    }

    pub fn fg(&self, kind: ThemeFgKind) -> Oklaba {
        match kind {
            ThemeFgKind::Primary => self.fg_primary,
            ThemeFgKind::Secondary => self.fg_secondary,
            ThemeFgKind::Tertiary => self.fg_tertiary,
        }
    }

    pub fn fg_inverse(&self, kind: ThemeFgKind) -> Oklaba {
        match kind {
            ThemeFgKind::Primary => self.fg_inverse_primary,
            ThemeFgKind::Secondary => self.fg_inverse_secondary,
            ThemeFgKind::Tertiary => self.fg_inverse_tertiary,
        }
    }

    pub fn fg_for_bg(
        &self,
        kind: ThemeFgKind,
        bg: impl IntoColor<Oklaba>,
    ) -> Oklaba {
        let bg = bg.into_color();

        let (fg, fg_inverse) = match kind {
            ThemeFgKind::Primary => (self.fg_primary, self.fg_inverse_primary),
            ThemeFgKind::Secondary => (self.fg_secondary, self.fg_inverse_secondary),
            ThemeFgKind::Tertiary => (self.fg_tertiary, self.fg_inverse_tertiary),
        };

        if contrast_ratio(fg, bg) >= contrast_ratio(fg_inverse, bg) {
            fg
        } else {
            fg_inverse
        }
    }

    pub fn interact_color(
        &self,
        color: impl IntoColor<Oklaba>,
        factor: f32,
    ) -> Oklaba {
        let color = color.into_color();

        let target = if contrast_ratio(color, self.bg_primary)
            <= contrast_ratio(color, self.fg_primary)
        {
            self.fg_primary
        } else {
            self.bg_primary
        };

        color.mix(target, factor)
    }

    pub fn hover_color(&self, color: impl IntoColor<Oklaba>) -> Oklaba {
        self.interact_color(color, Self::HOVER_COLOR_MIX)
    }

    pub fn active_color(&self, color: impl IntoColor<Oklaba>) -> Oklaba {
        self.interact_color(color, Self::ACTIVE_COLOR_MIX)
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
            ThemeTextSizeKind::Xs => self.text_size_xs,
            ThemeTextSizeKind::Sm => self.text_size_sm,
            ThemeTextSizeKind::Base => self.text_size_base,
            ThemeTextSizeKind::Lg => self.text_size_lg,
            ThemeTextSizeKind::Xl => self.text_size_xl,
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
            ThemeRadiiKind::Xs => self.radii_xs,
            ThemeRadiiKind::Sm => self.radii_sm,
            ThemeRadiiKind::Md => self.radii_md,
            ThemeRadiiKind::Lg => self.radii_lg,
            ThemeRadiiKind::Xl => self.radii_xl,
            ThemeRadiiKind::X2l => self.radii_2xl,
            ThemeRadiiKind::X3l => self.radii_3xl,
            ThemeRadiiKind::X4l => self.radii_4xl,
        }
    }

    pub fn size(&self, kind: ThemeSizeKind) -> Pixels {
        match kind {
            ThemeSizeKind::Xs => self.size_xs,
            ThemeSizeKind::Sm => self.size_sm,
            ThemeSizeKind::Md => self.size_md,
            ThemeSizeKind::Lg => self.size_lg,
            ThemeSizeKind::Xl => self.size_xl,
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
    Xs,
    Sm,
    Base,
    Lg,
    Xl,
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
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    X2l,
    X3l,
    X4l,
}

#[derive(Clone, Copy)]
pub enum ThemeSizeKind {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}
