use std::rc::Rc;

use gpui::{
    App, DurationWithEasing, ElementId, InteractiveElement, IntoElement, Lerp,
    ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, div, ease_in_out, linear_color_stop,
    linear_gradient, millis, prelude::FluentBuilder, selectors::class,
};
use palette::WithAlpha;
use tesserae_base::switch::{BaseSwitch, BaseSwitchThumb, OnChange};
use tesserae_theme::Theme;
use tesserae_utils::{PerceptualColor, StyledElement};

#[derive(IntoElement)]
pub struct Switch {
    element_id: ElementId,
    checked: bool,
    disabled: bool,
    drag_threshold: Pixels,
    aria_label: Option<SharedString>,
    on_change: Option<OnChange>,
    style: StyleRefinement,
}

impl Switch {
    pub fn new(element_id: impl Into<ElementId>) -> Self {
        Self {
            element_id: element_id.into(),
            checked: false,
            disabled: false,
            drag_threshold: BaseSwitch::DRAG_THRESHOLD,
            aria_label: None,
            on_change: None,
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

    pub fn drag_threshold(mut self, threshold: Pixels) -> Self {
        self.drag_threshold = threshold;

        self
    }

    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());

        self
    }

    pub fn on_change(
        mut self,
        on_change: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(on_change));

        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::read_global(cx);
        let checked = self.checked;
        let disabled = self.disabled;

        let thumb_height = theme.size_3xl.to_pixels(window.rem_size());
        let thumb_width = thumb_height * theme.thumb_ratio.as_f32();
        let padding = theme.padding_sm.to_pixels(window.rem_size());

        let width = padding * 2 + thumb_width + thumb_height;
        let height = thumb_height + padding * 2;

        BaseSwitch::new(self.element_id.clone())
            .checked(checked)
            .disabled(disabled)
            .drag_threshold(self.drag_threshold)
            .min_w(width)
            .min_h(height)
            .p(padding)
            .cursor_pointer()
            .rounded_full()
            .rounded_smoothing_1()
            .bg(theme.bg_secondary)
            .inset_ring_1()
            .inset_ring_color(theme.bg_tertiary.lerp(&theme.bg_quaternary, 0.5))
            .select(class(BaseSwitch::CHECKED_CLASS), |refinement| {
                refinement
                    .select_children(class(BaseSwitchThumb::CLASS), |refinement| {
                        refinement.right(padding).bg(theme
                            .accent_primary
                            .best_contrast([theme.bg_secondary, theme.fg_primary]))
                    })
                    .select_children(class("overlay"), |refinement| {
                        refinement.opacity(1.)
                    })
            })
            .when_else(
                !disabled,
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
                },
                |this| this.cursor_not_allowed().opacity(0.48),
            )
            .transitions(|transitions| {
                transitions.bg(millis(200).with_easing(ease_in_out))
            })
            .child(
                div()
                    .id((self.element_id.clone(), "overlay"))
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
                BaseSwitchThumb::new((self.element_id, "thumb"))
                    .absolute()
                    .w(thumb_width)
                    .h(thumb_height)
                    .rounded_full()
                    .rounded_smoothing_1()
                    .bg(theme.fg_primary)
                    .transitions(|transitions| {
                        transitions
                            .bg(millis(200).with_easing(ease_in_out))
                            .right(millis(100).with_easing(ease_in_out))
                    }),
            )
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_change, |this, on_change| {
                this.on_change(move |checked, window, cx| {
                    on_change(checked, window, cx)
                })
            })
            .refine_style(&self.style)
    }
}

impl Styled for Switch {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
