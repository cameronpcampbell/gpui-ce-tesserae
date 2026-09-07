use std::rc::Rc;

use gpui::{
    App, DurationWithEasing, ElementId, InteractiveElement, IntoElement, Lerp,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, div, ease_in_out, linear_color_stop,
    linear_gradient, millis, prelude::FluentBuilder, selectors::class,
};
use palette::WithAlpha;
use tesserae_base::switch::{BaseSwitch, BaseSwitchThumb};
use tesserae_theme::Theme;
use tesserae_utils::{PerceptualColor, StyledElement, kinds};

type OnClick = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    checked: bool,
    disabled: bool,
    aria_label: Option<SharedString>,
    on_click: Option<OnClick>,
    style: StyleRefinement,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            disabled: false,
            aria_label: None,
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

    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::read_global(cx);
        let checked = self.checked;
        let disabled = self.disabled;

        let knob_height = theme.size_lg.to_pixels(window.rem_size());
        let knob_width = knob_height * theme.knob_ratio.as_f32();
        let padding = theme.padding_sm.to_pixels(window.rem_size());

        let width = padding * 2 + knob_width + knob_height;
        let height = knob_height + padding * 2;

        BaseSwitch::new(self.id.clone())
            .checked(checked)
            .disabled(disabled)
            .cursor_pointer()
            .w(width)
            .h(height)
            .p(padding)
            .rounded_full()
            .rounded_smoothing_1()
            .bg(theme.bg_secondary)
            .inset_ring_1()
            .inset_ring_color(theme.bg_tertiary.lerp(&theme.bg_quaternary, 0.5))
            .apply_kind(SwitchStateKind::Disabled, theme)
            .select(class(BaseSwitch::CHECKED_CLASS), |style| {
                style.apply_kind(SwitchStateKind::Enabled, theme)
            })
            .select(class(BaseSwitch::DISABLED_CLASS), |style| {
                style.cursor_not_allowed().opacity(0.48)
            })
            .transitions(|transitions| {
                transitions.bg(millis(200).with_easing(ease_in_out))
            })
            .when(!disabled, |this| {
                this.hover(|style| {
                    style
                        .bg(theme.hover_feedback(theme.bg_secondary))
                        .select_children(class("overlay"), |style| {
                            style.bg(theme.hover_feedback(theme.accent_primary))
                        })
                })
                .active(|style| {
                    style
                        .bg(theme.active_feedback(theme.bg_secondary))
                        .select_children(class("overlay"), |style| {
                            style.bg(theme.active_feedback(theme.accent_primary))
                        })
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
                        transitions.bg(millis(200).with_easing(ease_in_out))
                    }),
            )
            .thumb(
                BaseSwitchThumb::new((self.id, "thumb"))
                    .absolute()
                    .w(knob_width)
                    .h(knob_height)
                    .rounded_full()
                    .rounded_smoothing_1()
                    .select(class(BaseSwitchThumb::CHECKED_CLASS), |style| {
                        style.right(padding)
                    })
                    .transitions(|transitions| {
                        transitions
                            .bg(millis(200).with_easing(ease_in_out))
                            .right(millis(100).with_easing(ease_in_out))
                    }),
            )
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_click, |this, on_click| {
                this.on_change(move |checked, window, cx| {
                    on_click(checked, window, cx)
                })
            })
            .refine(self.style)
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
            .select_children(class(BaseSwitchThumb::CLASS), |style| style.bg(theme.fg_primary))
    },

    Enabled (this, theme) => {
        this
            .select_children(class(BaseSwitchThumb::CLASS), |style| {
                style.bg(theme
                    .accent_primary
                    .best_contrast([theme.bg_secondary, theme.fg_primary]))
            })
            .select_descendants(class("overlay"), |style| style.opacity(1.))
    },
});

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{
        AppContext, Bounds, Context, Entity, InteractiveElement, IntoElement,
        Modifiers, MouseButton, ParentElement, Pixels, Render, Styled,
        TestAppContext, VisualTestContext, Window, div, point, px,
    };
    use tesserae_theme::{ThemeConfig, ThemeSet, ThemeSetKind};

    use super::Switch;

    const SWITCH_HOST: &str = "switch-host";

    struct SwitchTestView {
        checked: bool,
        disabled: bool,
        changes: Vec<bool>,
    }

    impl Render for SwitchTestView {
        fn render(
            &mut self,
            _window: &mut Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            div().size_full().child(
                div()
                    .absolute()
                    .left(px(40.))
                    .top(px(40.))
                    .size(px(100.))
                    .debug_selector(|| SWITCH_HOST.to_owned())
                    .child(
                        Switch::new("switch-under-test")
                            .checked(self.checked)
                            .disabled(self.disabled)
                            .on_click(cx.listener(|this, checked, _window, cx| {
                                this.checked = *checked;
                                this.changes.push(*checked);
                                cx.notify();
                            })),
                    ),
            )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        checked: bool,
        disabled: bool,
    ) -> (Entity<SwitchTestView>, &mut VisualTestContext) {
        cx.update(|cx| {
            ThemeSet::set_global(cx, ThemeSet::generate(ThemeConfig::default()));
            ThemeSetKind::set_global(cx, ThemeSetKind::Dark);
        });

        cx.add_window_view(move |_window, _cx| SwitchTestView {
            checked,
            disabled,
            changes: Vec::new(),
        })
    }

    fn host_bounds(cx: &mut VisualTestContext) -> Bounds<Pixels> {
        cx.debug_bounds(SWITCH_HOST)
            .expect("switch host should be rendered")
    }

    fn state(
        view: &Entity<SwitchTestView>,
        cx: &VisualTestContext,
    ) -> (bool, Vec<bool>) {
        cx.read_entity(view, |view, _cx| (view.checked, view.changes.clone()))
    }

    #[gpui::test]
    fn styled_switch_delegates_interaction_and_disabled_state(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, false, false);
        let bounds = host_bounds(cx);
        let inside = point(bounds.left() + px(20.), bounds.top() + px(10.));

        cx.simulate_click(inside, Modifiers::none());
        assert_eq!(state(&view, cx), (true, vec![true]));

        let outside_left = point(bounds.left() - px(20.), inside.y);
        cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(outside_left, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(outside_left, MouseButton::Left, Modifiers::none());
        assert_eq!(state(&view, cx), (false, vec![true, false]));

        view.update(cx, |view, cx| {
            view.disabled = true;
            cx.notify();
        });
        let outside_right = point(bounds.right() + px(20.), inside.y);
        cx.simulate_click(inside, Modifiers::none());
        cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(outside_right, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(outside_right, MouseButton::Left, Modifiers::none());
        assert_eq!(state(&view, cx), (false, vec![true, false]));

        view.update(cx, |view, cx| {
            view.disabled = false;
            cx.notify();
        });
        cx.simulate_click(inside, Modifiers::none());
        assert_eq!(state(&view, cx), (true, vec![true, false, true]));
    }
}
