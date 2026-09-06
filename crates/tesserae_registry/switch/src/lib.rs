use std::rc::Rc;

use gpui::{
    App, DurationWithEasing, ElementId, InteractiveElement, IntoElement, Lerp,
    ParentElement, RenderOnce, StatefulInteractiveElement, StyleRefinement, Styled,
    Window, div, ease_in_out, linear_color_stop, linear_gradient, millis,
    prelude::FluentBuilder, selectors::class,
};
use palette::WithAlpha;
use tesserae_utils::{PerceptualColor, StyledElement, kinds, use_focus_handle};

use tesserae_theme::Theme;

#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    checked: bool,
    disabled: bool,
    on_click: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    style: StyleRefinement,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            disabled: false,
            on_click: None,
            style: StyleRefinement::default(),
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        on_click: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }
}

impl RenderOnce for Switch {
    fn render(
        self,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> impl gpui::IntoElement {
        let _focus_handle = use_focus_handle(self.id.clone(), window, cx, None);

        let checked = self.checked;

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
            .apply_kind(SwitchStateKind::from_bool(checked), theme)
            .transitions(|transitions| {
                transitions.bg(millis(200).with_easing(ease_in_out))
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
                        transitions.bg(millis(200).with_easing(ease_in_out))
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
                    .when(checked, |this| this.right(padding))
                    .transitions(|transitions| {
                        transitions
                            .bg(millis(200).with_easing(ease_in_out))
                            .right(millis(100).with_easing(ease_in_out))
                    }),
            )
            .when_else(
                !self.disabled,
                |this| {
                    this.hover(|refinement| {
                        refinement
                            .bg(theme.hover_feedback(theme.bg_secondary))
                            .select_children(class("overlay"), |refinement| {
                                refinement
                                    .bg(theme.hover_feedback(theme.accent_primary))
                            })
                    })
                    .active(|refinement| {
                        refinement
                            .bg(theme.active_feedback(theme.bg_secondary))
                            .select_children(class("overlay"), |refinement| {
                                refinement
                                    .bg(theme.active_feedback(theme.accent_primary))
                            })
                    })
                    .when_some(
                        self.on_click,
                        |this, on_click| {
                            this.on_click(move |_event, window, cx| {
                                on_click(&!checked, window, cx);
                            })
                        },
                    )
                },
                |this| this.cursor_not_allowed().opacity(0.48),
            )
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
            .select_descendants(class("overlay"), |refinement| refinement.opacity(1.))
    },
});

impl SwitchStateKind {
    fn from_bool(value: bool) -> Self {
        if value { Self::Enabled } else { Self::Disabled }
    }
}
