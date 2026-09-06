use std::rc::Rc;

use gpui::{
    App, ClickEvent, DispatchPhase, DurationWithEasing, ElementId, Entity,
    InteractiveElement, IntoElement, Lerp, MouseButton, MouseMoveEvent,
    ParentElement, Pixels, RenderOnce, StatefulInteractiveElement, StyleRefinement,
    Styled, Window, canvas, div, ease_in_out, linear_color_stop, linear_gradient,
    millis, prelude::FluentBuilder, px, selectors::class,
};
use palette::WithAlpha;
use tesserae_utils::{PerceptualColor, StyledElement, kinds, use_focus_handle};

use tesserae_theme::Theme;

const DRAG_THRESHOLD: Pixels = px(10.);
type OnClick = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

#[derive(Clone, Copy, Default)]
struct DragState {
    start_x: Option<Pixels>,
    dragged_to: Option<bool>,
}

impl DragState {
    fn start(&mut self, x: Pixels) {
        self.start_x = Some(x);
        self.dragged_to = None;
    }

    fn update(&mut self, x: Pixels) -> bool {
        let Some(start_x) = self.start_x else {
            return false;
        };

        let previous = self.dragged_to;
        let delta = x - start_x;

        if delta > DRAG_THRESHOLD {
            self.dragged_to = Some(true);
        } else if delta < -DRAG_THRESHOLD {
            self.dragged_to = Some(false);
        }

        self.dragged_to != previous
    }

    fn finish(
        &mut self,
        x: Pixels,
        checked: bool,
        released_inside: bool,
    ) -> Option<bool> {
        self.update(x);

        let next = match self.dragged_to {
            Some(next) if next != checked => Some(next),
            Some(_) => None,
            None if released_inside => Some(!checked),
            None => None,
        };

        *self = Self::default();
        next
    }
}

fn drag_tracker(drag_state: Entity<DragState>) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |_, _, window, _| {
            window.on_mouse_event(
                move |event: &MouseMoveEvent, phase, _window, cx| {
                    if phase != DispatchPhase::Capture || !event.dragging() {
                        return;
                    }

                    drag_state.update(cx, |state, cx| {
                        if state.update(event.position.x) {
                            cx.notify();
                        }
                    });
                },
            );
        },
    )
    .absolute()
    .inset_0()
}

#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    checked: bool,
    disabled: bool,
    on_click: Option<OnClick>,
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
        let drag_state = window.use_keyed_state(
            (self.id.clone(), "state:drag"),
            cx,
            |_window, _cx| DragState::default(),
        );
        let can_change = !self.disabled && self.on_click.is_some();
        let drag_active = drag_state.read(cx).start_x.is_some();

        if !can_change && drag_active {
            drag_state.update(cx, |state, _cx| *state = DragState::default());
        }

        let effective_checked = if can_change {
            drag_state.read(cx).dragged_to.unwrap_or(checked)
        } else {
            checked
        };

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
            .apply_kind(SwitchStateKind::from_bool(effective_checked), theme)
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
                    .when(effective_checked, |this| this.right(padding))
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
                            let drag_state_on_down = drag_state.clone();
                            let drag_state_on_up = drag_state.clone();
                            let on_mouse_up = on_click.clone();

                            this.child(drag_tracker(drag_state))
                                .on_mouse_down_all(
                                    move |event, phase, hitbox, window, cx| {
                                        if phase == DispatchPhase::Bubble
                                            && event.button == MouseButton::Left
                                            && hitbox.is_hovered(window)
                                        {
                                            drag_state_on_down.update(
                                                cx,
                                                |state, _cx| {
                                                    state.start(event.position.x)
                                                },
                                            );
                                        }
                                    },
                                )
                                .on_mouse_up_all(
                                    move |event, phase, hitbox, window, cx| {
                                        if phase != DispatchPhase::Capture
                                            || event.button != MouseButton::Left
                                        {
                                            return;
                                        }

                                        let next = drag_state_on_up.update(
                                            cx,
                                            |state, cx| {
                                                if state.start_x.is_some() {
                                                    let next = state.finish(
                                                        event.position.x,
                                                        checked,
                                                        hitbox.bounds.contains(
                                                            &event.position,
                                                        ),
                                                    );
                                                    cx.notify();
                                                    next
                                                } else {
                                                    None
                                                }
                                            },
                                        );

                                        if let Some(next) = next {
                                            on_mouse_up(&next, window, cx);
                                        }
                                    },
                                )
                                .on_click(move |event, window, cx| {
                                    if !matches!(event, ClickEvent::Mouse(_)) {
                                        on_click(&!checked, window, cx);
                                    }
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
