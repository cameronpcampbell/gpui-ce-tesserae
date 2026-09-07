use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, DispatchPhase, Div, ElementId, Entity, FocusHandle,
    InteractiveElement, Interactivity, IntoElement, MouseButton, MouseMoveEvent,
    ParentElement, Pixels, RenderOnce, Role, SharedString, Stateful,
    StatefulInteractiveElement, StyleRefinement, Styled, Toggled, Window, canvas,
    div, prelude::FluentBuilder, px,
};
use tesserae_utils::{StyledElement, use_focus_handle};

type OnChange = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// An unstyled, controlled switch.
///
/// `BaseSwitch` owns pointer, drag, keyboard, focus, and accessibility
/// behavior. Callers own its layout and appearance through GPUI styles and
/// selectors.
#[derive(IntoElement)]
pub struct BaseSwitch {
    id: ElementId,
    base: Stateful<Div>,
    style: StyleRefinement,
    checked: bool,
    disabled: bool,
    children: Vec<AnyElement>,
    on_change: Option<OnChange>,
    aria_label: Option<SharedString>,
}

impl BaseSwitch {
    pub const CHECKED_CLASS: &'static str = "checked";
    pub const UNCHECKED_CLASS: &'static str = "unchecked";
    pub const DISABLED_CLASS: &'static str = "disabled";

    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();

        Self {
            id: id.clone(),
            base: div().id(id),
            style: StyleRefinement::default(),
            checked: false,
            disabled: false,
            children: Vec::new(),
            on_change: None,
            aria_label: None,
        }
    }

    /// Sets the application-controlled checked value.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Sets whether the switch ignores user interaction.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Handles activation with the next checked value.
    pub fn on_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Sets the name exposed to accessibility clients.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    fn focus_handle(&self, window: &mut Window, cx: &mut App) -> FocusHandle {
        window
            .use_keyed_state(
                (self.id.clone(), "state:focus_handle"),
                cx,
                |_window, cx| cx.focus_handle().tab_stop(true),
            )
            .read(cx)
            .clone()
    }
}

impl Styled for BaseSwitch {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for BaseSwitch {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl InteractiveElement for BaseSwitch {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for BaseSwitch {}

impl RenderOnce for BaseSwitch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.checked;
        let disabled = self.disabled;
        let focus_handle = use_focus_handle(self.id.clone(), window, cx, None);

        let drag = BaseSwitchDrag::new(&self.id, window, cx);
        let on_change = if disabled { None } else { self.on_change };

        if on_change.is_none() {
            drag.cancel(cx);
        }

        let effective_checked = if on_change.is_some() {
            drag.effective_checked(checked, cx)
        } else {
            checked
        };

        self.base
            .refine_style(&self.style)
            .when(effective_checked, |this| this.class(Self::CHECKED_CLASS))
            .when(!effective_checked, |this| this.class(Self::UNCHECKED_CLASS))
            .when(disabled, |this| this.class(Self::DISABLED_CLASS))
            .role(Role::Switch)
            .aria_toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            })
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when(!disabled, |this| this.track_focus(&focus_handle))
            .when(disabled, |this| {
                this.on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
            })
            .when_some(on_change, |this, on_change| {
                let keyboard_change = on_change.clone();

                drag.install(this, checked, on_change).on_click(
                    move |event, window, cx| {
                        if !matches!(event, ClickEvent::Mouse(_)) {
                            keyboard_change(&!checked, window, cx);
                        }
                    },
                )
            })
            .children(self.children)
    }
}

/// The unstyled thumb rendered by a [`BaseSwitch`].
///
/// The parent switch projects its effective checked and disabled states as
/// classes before rendering the thumb. Callers provide all visual styles.
#[derive(IntoElement)]
pub struct BaseSwitchThumb {
    base: Stateful<Div>,
    style: StyleRefinement,
    children: Vec<AnyElement>,
}

impl BaseSwitchThumb {
    pub const CLASS: &'static str = "thumb";

    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div().id(id),
            style: StyleRefinement::default(),
            children: Vec::new(),
        }
    }
}

impl RenderOnce for BaseSwitchThumb {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.base
            .class(Self::CLASS)
            .refine_style(&self.style)
            .children(self.children)
    }
}

impl Styled for BaseSwitchThumb {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for BaseSwitchThumb {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl InteractiveElement for BaseSwitchThumb {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for BaseSwitchThumb {}

struct BaseSwitchDrag {
    state: Entity<DragState>,
}

impl BaseSwitchDrag {
    fn new(id: &ElementId, window: &mut Window, cx: &mut App) -> Self {
        let state = window.use_keyed_state(
            (id.clone(), "state:drag"),
            cx,
            |_window, _cx| DragState::default(),
        );

        Self { state }
    }

    fn effective_checked(&self, checked: bool, cx: &App) -> bool {
        self.state.read(cx).dragged_to.unwrap_or(checked)
    }

    fn cancel(&self, cx: &mut App) {
        if self.state.read(cx).start_x.is_some() {
            self.state
                .update(cx, |state, _cx| *state = DragState::default());
        }
    }

    fn install(
        self,
        base: Stateful<Div>,
        checked: bool,
        on_change: OnChange,
    ) -> Stateful<Div> {
        let drag_state_on_down = self.state.clone();
        let drag_state_on_up = self.state.clone();

        base.child(self.tracker())
            .on_mouse_down_all(move |event, phase, hitbox, window, cx| {
                if phase == DispatchPhase::Bubble
                    && event.button == MouseButton::Left
                    && hitbox.is_hovered(window)
                {
                    drag_state_on_down
                        .update(cx, |state, _cx| state.start(event.position.x));
                }
            })
            .on_mouse_up_all(move |event, phase, hitbox, window, cx| {
                if phase != DispatchPhase::Capture
                    || event.button != MouseButton::Left
                {
                    return;
                }

                let next = drag_state_on_up.update(cx, |state, cx| {
                    state.start_x?;

                    let next = state.finish(
                        event.position.x,
                        checked,
                        hitbox.bounds.contains(&event.position),
                    );
                    cx.notify();
                    next
                });

                if let Some(next) = next {
                    on_change(&next, window, cx);
                }
            })
    }

    fn tracker(self) -> impl IntoElement {
        canvas(
            |_, _, _| (),
            move |_, _, window, _| {
                window.on_mouse_event(
                    move |event: &MouseMoveEvent, phase, _window, cx| {
                        if phase != DispatchPhase::Capture || !event.dragging() {
                            return;
                        }

                        self.state.update(cx, |state, cx| {
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
}

const DRAG_THRESHOLD: Pixels = px(10.);

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

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use std::sync::{Arc, Mutex};

    use gpui::{
        AppContext, Bounds, Context, Element as _, Entity, InteractiveElement,
        IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, MouseButton,
        ParentElement, Pixels, Render, RenderOnce, StatefulInteractiveElement,
        Styled, TestAppContext, VisualTestContext, Window, accesskit, canvas, div,
        point, prelude::FluentBuilder, px, selectors::class,
    };

    use super::{BaseSwitch, BaseSwitchThumb};

    const SWITCH: &str = "base-switch";
    const THUMB: &str = "base-switch-thumb";

    struct SwitchHarness {
        checked: bool,
        disabled: bool,
        has_handler: bool,
        accept_changes: bool,
        changes: Vec<bool>,
        parent_clicks: usize,
    }

    impl Render for SwitchHarness {
        fn render(
            &mut self,
            _window: &mut Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            div()
                .id("switch-parent")
                .tab_group()
                .size_full()
                .on_click(cx.listener(|this, _, _, _| this.parent_clicks += 1))
                .child(
                    BaseSwitch::new("switch-under-test")
                        .checked(self.checked)
                        .disabled(self.disabled)
                        .absolute()
                        .left(px(40.))
                        .top(px(40.))
                        .w(px(100.))
                        .h(px(30.))
                        .select(class(BaseSwitch::CHECKED_CLASS), |style| {
                            style.w(px(120.))
                        })
                        .select(class(BaseSwitch::DISABLED_CLASS), |style| {
                            style.h(px(20.))
                        })
                        .debug_selector(|| SWITCH.to_owned())
                        .thumb(
                            BaseSwitchThumb::new("thumb-under-test")
                                .w(px(10.))
                                .h(px(10.))
                                .select(
                                    class(BaseSwitchThumb::CHECKED_CLASS),
                                    |style| style.w(px(20.)),
                                )
                                .select(
                                    class(BaseSwitchThumb::DISABLED_CLASS),
                                    |style| style.h(px(5.)),
                                )
                                .debug_selector(|| THUMB.to_owned()),
                        )
                        .when(self.has_handler, |this| {
                            this.on_change(cx.listener(
                                |this, checked, _window, cx| {
                                    this.changes.push(*checked);
                                    if this.accept_changes {
                                        this.checked = *checked;
                                    }
                                    cx.notify();
                                },
                            ))
                        }),
                )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        checked: bool,
        disabled: bool,
        accept_changes: bool,
    ) -> (Entity<SwitchHarness>, &mut VisualTestContext) {
        cx.add_window_view(move |_window, _cx| SwitchHarness {
            checked,
            disabled,
            has_handler: true,
            accept_changes,
            changes: Vec::new(),
            parent_clicks: 0,
        })
    }

    fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
        cx.debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} should be rendered"))
    }

    fn state(
        view: &Entity<SwitchHarness>,
        cx: &VisualTestContext,
    ) -> (bool, Vec<bool>, usize) {
        cx.read_entity(view, |view, _cx| {
            (view.checked, view.changes.clone(), view.parent_clicks)
        })
    }

    fn activate_key(cx: &mut VisualTestContext, key: &str) {
        let keystroke = Keystroke::parse(key).unwrap();
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
    }

    #[gpui::test]
    fn pointer_gestures_project_state_and_report_changes_once(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, false, false, true);
        let initial = bounds(cx, SWITCH);
        let inside = point(initial.left() + px(10.), initial.top() + px(10.));

        assert_eq!(initial.size.width, px(100.));
        assert_eq!(bounds(cx, THUMB).size.width, px(10.));

        let outside_right = point(initial.right() + px(50.), inside.y);
        cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(outside_right, MouseButton::Left, Modifiers::none());

        assert!(!state(&view, cx).0);
        assert!(state(&view, cx).1.is_empty());
        assert_eq!(bounds(cx, SWITCH).size.width, px(120.));
        assert_eq!(bounds(cx, THUMB).size.width, px(20.));

        cx.simulate_mouse_up(outside_right, MouseButton::Left, Modifiers::none());
        assert!(state(&view, cx).0);
        assert_eq!(state(&view, cx).1, vec![true]);

        let outside_left = point(initial.left() - px(30.), inside.y);
        cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(outside_left, MouseButton::Left, Modifiers::none());
        assert_eq!(bounds(cx, SWITCH).size.width, px(100.));
        assert_eq!(bounds(cx, THUMB).size.width, px(10.));
        cx.simulate_mouse_up(outside_left, MouseButton::Left, Modifiers::none());
        assert_eq!(state(&view, cx).1, vec![true, false]);

        cx.simulate_click(inside, Modifiers::none());
        let small_move = point(inside.x + px(5.), inside.y);
        cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(small_move, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(small_move, MouseButton::Left, Modifiers::none());

        let outside_below = point(inside.x, initial.bottom() + px(20.));
        cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(outside_below, MouseButton::Left, Modifiers::none());

        assert!(!state(&view, cx).0);
        assert_eq!(state(&view, cx).1, vec![true, false, true, false]);
    }

    #[gpui::test]
    fn controlled_state_changes_only_when_rendered_back(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, false, false, false);
        let switch = bounds(cx, SWITCH);
        let inside = point(switch.left() + px(10.), switch.top() + px(10.));

        cx.simulate_click(inside, Modifiers::none());
        cx.simulate_click(inside, Modifiers::none());
        activate_key(cx, "enter");
        activate_key(cx, "space");

        assert!(!state(&view, cx).0);
        assert_eq!(state(&view, cx).1, vec![true, true, true, true]);
        assert_eq!(bounds(cx, SWITCH).size.width, px(100.));
    }

    #[gpui::test]
    fn disabled_state_is_inert_and_cancels_an_active_drag(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, false, true, true);
        let switch = bounds(cx, SWITCH);
        let inside = point(switch.left() + px(10.), switch.top() + px(10.));
        let outside_right = point(switch.right() + px(50.), inside.y);

        assert_eq!(switch.size.height, px(20.));
        assert_eq!(bounds(cx, THUMB).size.height, px(5.));

        cx.simulate_click(inside, Modifiers::none());
        cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(outside_right, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(outside_right, MouseButton::Left, Modifiers::none());
        activate_key(cx, "enter");
        assert_eq!(state(&view, cx), (false, Vec::new(), 0));

        view.update(cx, |view, cx| {
            view.disabled = false;
            cx.notify();
        });
        cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(outside_right, MouseButton::Left, Modifiers::none());
        assert_eq!(bounds(cx, SWITCH).size.width, px(120.));

        view.update(cx, |view, cx| {
            view.disabled = true;
            cx.notify();
        });
        assert_eq!(bounds(cx, SWITCH).size.width, px(100.));
        cx.simulate_mouse_up(outside_right, MouseButton::Left, Modifiers::none());

        assert!(!state(&view, cx).0);
        assert!(state(&view, cx).1.is_empty());
    }

    #[gpui::test]
    fn accessibility_exposes_the_switch_value_and_action(cx: &mut TestAppContext) {
        type Captured = Arc<Mutex<Option<(accesskit::Node, accesskit::Node)>>>;

        struct AccessibilityProbe {
            captured: Captured,
        }

        impl Render for AccessibilityProbe {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let captured = self.captured.clone();
                canvas(
                    move |_, window, cx| {
                        let mut info = |switch: BaseSwitch| {
                            let mut node =
                                accesskit::Node::new(accesskit::Role::Switch);
                            switch
                                .render(window, cx)
                                .into_element()
                                .write_a11y_info(&mut node);
                            node
                        };

                        let enabled = info(
                            BaseSwitch::new("enabled")
                                .checked(true)
                                .aria_label("Notifications")
                                .on_change(|_, _, _| {}),
                        );
                        let disabled = info(
                            BaseSwitch::new("disabled")
                                .disabled(true)
                                .aria_label("Notifications")
                                .on_change(|_, _, _| {}),
                        );
                        *captured.lock().unwrap() = Some((enabled, disabled));
                    },
                    |_, _, _, _| {},
                )
            }
        }

        let captured: Captured = Arc::new(Mutex::new(None));
        let result = captured.clone();
        let (_, cx) =
            cx.add_window_view(move |_, _| AccessibilityProbe { captured });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let (enabled, disabled) = result.lock().unwrap().take().unwrap();

        assert_eq!(enabled.role(), accesskit::Role::Switch);
        assert_eq!(enabled.label(), Some("Notifications"));
        assert_eq!(enabled.toggled(), Some(accesskit::Toggled::True));
        assert!(enabled.supports_action(accesskit::Action::Click));

        assert_eq!(disabled.role(), accesskit::Role::Switch);
        assert_eq!(disabled.toggled(), Some(accesskit::Toggled::False));
        assert!(!disabled.supports_action(accesskit::Action::Click));
    }
}
