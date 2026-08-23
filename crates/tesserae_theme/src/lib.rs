use gpui::Rgba;
use palette::color_difference::Wcag21RelativeContrast;

mod generate;
use generate::generate_theme;

#[derive(Clone)]
pub struct ThemeSet {
    light: Theme,
    dark: Theme,
}

impl ThemeSet {
    pub fn from_accents(config: ThemeAccentsConfig) -> ThemeSet {
        Self {
            light: generate_theme(&config, ThemeSetKind::Light),
            dark: generate_theme(&config, ThemeSetKind::Dark),
        }
    }
}

#[derive(Clone, Copy)]
pub enum ThemeSetKind {
    Light,
    Dark,
}

#[derive(Clone)]
pub struct ThemeAccentsConfig {
    pub accent_primary: Rgba,
    pub accent_caution: Rgba,
    pub accent_destruct: Rgba,
}

#[derive(Clone)]
pub struct Theme {
    pub bg_primary: Rgba,
    pub bg_secondary: Rgba,
    pub bg_tertiary: Rgba,
    pub bg_quaternary: Rgba,
    pub bg_quinary: Rgba,
    pub bg_senary: Rgba,

    pub fg_primary: Rgba,
    pub fg_secondary: Rgba,
    pub fg_tertiary: Rgba,

    pub fg_inverse_primary: Rgba,
    pub fg_inverse_secondary: Rgba,
    pub fg_inverse_tertiary: Rgba,

    pub accent_primary: Rgba,
    pub accent_secondary: Rgba,
    pub accent_caution: Rgba,
    pub accent_destruct: Rgba,
}

impl Theme {
    pub fn bg(&self, kind: ThemeBgKind) -> Rgba {
        match kind {
            ThemeBgKind::Primary => self.bg_primary,
            ThemeBgKind::Secondary => self.bg_secondary,
            ThemeBgKind::Tertiary => self.bg_tertiary,
            ThemeBgKind::Quaternary => self.bg_quaternary,
            ThemeBgKind::Quinary => self.bg_quinary,
            ThemeBgKind::Senary => self.bg_senary,
        }
    }

    pub fn fg(&self, kind: ThemeFgKind) -> Rgba {
        match kind {
            ThemeFgKind::Primary => self.fg_primary,
            ThemeFgKind::Secondary => self.fg_secondary,
            ThemeFgKind::Tertiary => self.fg_tertiary,
        }
    }

    pub fn fg_inverse(&self, kind: ThemeFgKind) -> Rgba {
        match kind {
            ThemeFgKind::Primary => self.fg_inverse_primary,
            ThemeFgKind::Secondary => self.fg_inverse_secondary,
            ThemeFgKind::Tertiary => self.fg_inverse_tertiary,
        }
    }

    pub fn fg_for_bg(&self, kind: ThemeFgKind, bg: Rgba) -> Rgba {
        let (fg, fg_inverse) = match kind {
            ThemeFgKind::Primary => (self.fg_primary, self.fg_inverse_primary),
            ThemeFgKind::Secondary => (self.fg_secondary, self.fg_inverse_secondary),
            ThemeFgKind::Tertiary => (self.fg_tertiary, self.fg_inverse_tertiary),
        };

        #[inline(always)]
        fn contrast_ratio(foreground: Rgba, background: Rgba) -> f32 {
            foreground.color.relative_contrast(background.color)
        }

        if contrast_ratio(fg, bg) >= contrast_ratio(fg_inverse, bg) {
            fg
        } else {
            fg_inverse
        }
    }

    pub fn accent(&self, kind: ThemeAccentKind) -> Rgba {
        match kind {
            ThemeAccentKind::Primary => self.accent_primary,
            ThemeAccentKind::Secondary => self.accent_secondary,
            ThemeAccentKind::Caution => self.accent_caution,
            ThemeAccentKind::Destruct => self.accent_destruct,
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
