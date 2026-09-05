use gpui::Background;

use super::*;
use crate::ThemeSet;

fn themes() -> Vec<Theme> {
    let purple = color_from_hex::<Oklaba>("6a41ffff").unwrap();
    [
        ThemeConfig::default(),
        ThemeConfig::default().base_fg(purple).base_bg(purple),
    ]
    .into_iter()
    .flat_map(|config| {
        let themes = ThemeSet::generate(config);
        [themes.light, themes.dark]
    })
    .collect()
}

fn fills(theme: &Theme) -> [Oklaba; 5] {
    [
        theme.accent_primary,
        theme.accent_secondary,
        theme.bg_secondary,
        theme.accent_caution,
        theme.accent_destruct,
    ]
}

fn displayed_lightness(color: Oklaba) -> f32 {
    let background = Background::from(color);
    let hsl = background.as_solid().unwrap();
    let rgb: Srgba = hsl.into_color();
    let displayed: Oklaba = rgb.into_color();
    displayed.color.l
}

#[test]
fn feedback_has_consistent_displayed_strength_across_theme_fills() {
    for theme in themes() {
        for base in fills(&theme) {
            for amount in [-0.08_f32, -0.04, 0.04, 0.08] {
                let before = displayed_lightness(base);
                let after = displayed_lightness(base.perceptual_feedback(amount));
                let step = after - before;

                // A lightness limit may shorten the step, but never reverse it.
                if before + amount <= 0.0 || before + amount >= 1.0 {
                    assert!(step.abs() <= amount.abs() + 1e-4);
                    assert!(step * amount >= -1e-4);
                } else {
                    assert!(
                        (step - amount).abs() < 1e-4,
                        "base {base:?}, amount {amount}: displayed step {step}"
                    );
                }
            }
        }
    }
}

#[test]
fn theme_feedback_has_ordered_states_and_moves_toward_the_contrasting_side() {
    for theme in themes() {
        // The neutral extremes exercise both directions in every theme.
        for base in fills(&theme).into_iter().chain([
            Oklaba::new(0.0, 0.0, 0.0, 1.0),
            Oklaba::new(1.0, 0.0, 0.0, 1.0),
        ]) {
            let rest = displayed_lightness(base);
            let hover = displayed_lightness(theme.hover_feedback(base));
            let active = displayed_lightness(theme.active_feedback(base));
            let hover_step = hover - rest;
            let active_step = active - rest;

            assert!(hover_step.abs() > 0.0, "base {base:?}");
            assert!(active_step * hover_step > 0.0, "base {base:?}");
            assert!(active_step.abs() >= hover_step.abs() - 1e-4);
            assert!(hover_step.abs() <= 0.04 + 1e-4);
            assert!(active_step.abs() <= 0.08 + 1e-4);

            // Away from the limits, active is twice as strong as hover,
            // with both measured from rest rather than chained together.
            if active > 1e-4 && active < 1.0 - 1e-4 {
                assert!((hover_step.abs() - 0.04).abs() < 1e-4);
                assert!((active_step.abs() - 0.08).abs() < 1e-4);
            }

            if base == theme.accent_secondary || base == theme.bg_secondary {
                if theme.bg_primary.color.l < 0.5 {
                    assert!(hover_step > 0.0);
                } else {
                    assert!(hover_step < 0.0);
                }
            } else if rest < 0.1 {
                assert!(hover_step > 0.0);
            } else if rest > 0.9 {
                assert!(hover_step < 0.0);
            }
        }
    }
}
