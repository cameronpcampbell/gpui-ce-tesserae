use gpui::{
    DurationWithEasing, ElementId, InteractiveElement, IntoElement, Lerp,
    ParentElement, RenderOnce, StatefulInteractiveElement, StyleRefinement, Styled,
    div, ease_in_out, linear_color_stop, linear_gradient, millis,
    prelude::FluentBuilder, selectors::class,
};
use palette::WithAlpha;
use tesserae_utils::{PerceptualColor, StyledElement, kinds, use_focus_handle};

use tesserae_theme::Theme;

#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    style: StyleRefinement,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for Switch {
    fn render(
        self,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> impl gpui::IntoElement {
        let _focus_handle = use_focus_handle(self.id.clone(), window, cx, None);

        let enabled_state = window.use_keyed_state(
            (self.id.clone(), "state:enabled"),
            cx,
            |_window, _cx| false,
        );
        let enabled = *enabled_state.read(cx);

        let theme = Theme::read_global(cx);

        let knob_height = theme.size_lg.to_pixels(window.rem_size());
        let knob_width = knob_height * theme.knob_ratio.0;

        let padding = theme.padding_sm.to_pixels(window.rem_size());

        let width = padding * 2 + knob_width + knob_height;
        let height = knob_height + padding * 2;

        div()
            .id(self.id.clone())
            .cursor_pointer()
            .w(width)
            .h(height)
            .p(padding)
            .rounded_full()
            .rounded_smoothing_1()
            .bg(theme.bg_secondary)
            .inset_ring_1()
            .inset_ring_color(theme.bg_tertiary.lerp(&theme.bg_quaternary, 0.5))
            .apply_kind(SwitchStateKind::from_bool(enabled), theme)
            .hover(|refinement| {
                refinement
                    .bg(theme.hover_feedback(theme.bg_secondary))
                    .select_descendants(class("overlay"), |refinement| {
                        refinement
                            .bg(theme.hover_feedback(theme.accent_primary))
                            .hidden()
                    })
            })
            .active(|refinement| {
                refinement
                    .bg(theme.active_feedback(theme.bg_secondary))
                    .select_children(class("overlay"), |refinement| {
                        refinement.bg(theme.active_feedback(theme.accent_primary))
                    })
            })
            .child(
                div()
                    .id((self.id.clone(), "overlay"))
                    .class("overlay")
                    .absolute()
                    .inset_0()
                    .rounded_full()
                    .rounded_smoothing_1()
                    .bg(theme.accent_primary)
                    .border_1()
                    .border_color(linear_gradient(
                        180.,
                        linear_color_stop(
                            theme.fg_primary.perceptual_alpha(0.08),
                            0.,
                        ),
                        linear_color_stop(theme.fg_primary.with_alpha(0.), 0.8),
                    ))
                    .opacity(0.)
                    .transitions(|transitions| {
                        transitions.bg(millis(100).with_easing(ease_in_out))
                    }),
            )
            .child(
                div()
                    .id((self.id.clone(), "knob"))
                    .class("knob")
                    .absolute()
                    .w(knob_width)
                    .h(knob_height)
                    .rounded_full()
                    .rounded_smoothing_1()
                    .when(enabled, |this| this.right(padding))
                    .transitions(|transitions| {
                        transitions
                            .bg(millis(100).with_easing(ease_in_out))
                            .right(millis(100).with_easing(ease_in_out))
                    }),
            )
            .on_click(move |_event, _window, cx| {
                enabled_state.update(cx, |enabled, cx| {
                    *enabled = !*enabled;
                    cx.notify();
                });
            })
    }
}

impl Styled for Switch {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

kinds!(pub SwitchStateKind<_, &Theme> {
    #[default]
    Disabled (this, theme) => {
        this
            .select_children(class("knob"), |refinement| refinement.bg(theme.fg_primary))
    },

    Enabled (this, theme) => {
        this
            .select_children(class("knob"), |refinement| {
                refinement.bg(theme
                    .accent_primary
                    .best_contrast([theme.bg_secondary, theme.fg_primary]))
            })
            .select_children(class("overlay"), |refinement| refinement.opacity(1.))
    },
});

impl SwitchStateKind {
    fn from_bool(value: bool) -> Self {
        if value { Self::Enabled } else { Self::Disabled }
    }
}
