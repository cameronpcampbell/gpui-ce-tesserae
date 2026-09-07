use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, InteractiveElement, Interactivity,
    IntoElement, MouseButton, ParentElement, RenderOnce, Role, SharedString,
    Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder,
};
use tesserae_utils::{StyledElement, use_focus_handle};

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// An unstyled button that owns activation, focus, and accessibility behavior.
///
/// Callers provide all layout and appearance through GPUI styles and selectors.
#[derive(IntoElement)]
pub struct BaseButton {
    id: ElementId,
    base: Stateful<Div>,
    style: StyleRefinement,
    disabled: bool,
    focusable_when_disabled: bool,
    children: Vec<AnyElement>,
    on_click: Option<ClickHandler>,
    aria_label: Option<SharedString>,
}

impl BaseButton {
    /// The class applied while the button is disabled.
    pub const DISABLED_CLASS: &'static str = "disabled";

    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();

        Self {
            id: id.clone(),
            base: div().id(id),
            style: StyleRefinement::default(),
            disabled: false,
            focusable_when_disabled: false,
            children: Vec::new(),
            on_click: None,
            aria_label: None,
        }
    }

    /// Sets whether the button ignores pointer and keyboard activation.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Keeps a disabled button in focus traversal and retains focus when its
    /// disabled state changes, while still preventing activation.
    pub fn focusable_when_disabled(mut self, focusable: bool) -> Self {
        self.focusable_when_disabled = focusable;
        self
    }

    /// Sets the name exposed to accessibility clients.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Handles pointer, Enter, and Space activation.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl Styled for BaseButton {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for BaseButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl InteractiveElement for BaseButton {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for BaseButton {}

impl RenderOnce for BaseButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = use_focus_handle(self.id.clone(), window, cx, None);
        let disabled = self.disabled;
        let focusable = !disabled || self.focusable_when_disabled;
        let on_click = if disabled { None } else { self.on_click };

        self.base
            .refine_style(&self.style)
            .when(disabled, |this| this.class(Self::DISABLED_CLASS))
            .role(Role::Button)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when(focusable, |this| this.track_focus(&focus_handle))
            .when(disabled, |this| {
                this.on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
            })
            .when_some(on_click, |this, on_click| {
                this.on_click(move |event, window, cx| {
                    on_click(event, window, cx);
                })
            })
            .children(self.children)
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use std::sync::{Arc, Mutex};

    use gpui::{
        AppContext, Bounds, Context, Element as _, Entity, InteractiveElement,
        IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, ParentElement,
        Pixels, Render, RenderOnce, StatefulInteractiveElement, Styled,
        TestAppContext, VisualTestContext, Window, accesskit, canvas, div, point,
        px, selectors::class,
    };

    use super::BaseButton;

    const BUTTON: &str = "base-button";

    struct ButtonHarness {
        disabled: bool,
        focusable_when_disabled: bool,
        clicks: Vec<bool>,
        parent_clicks: usize,
    }

    impl Render for ButtonHarness {
        fn render(
            &mut self,
            _window: &mut Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            div()
                .id("button-parent")
                .tab_group()
                .size_full()
                .on_click(cx.listener(|this, _, _, _| this.parent_clicks += 1))
                .child(
                    BaseButton::new("button-under-test")
                        .disabled(self.disabled)
                        .focusable_when_disabled(self.focusable_when_disabled)
                        .absolute()
                        .left(px(40.))
                        .top(px(40.))
                        .w(px(100.))
                        .h(px(30.))
                        .select(class(BaseButton::DISABLED_CLASS), |style| {
                            style.w(px(80.))
                        })
                        .debug_selector(|| BUTTON.to_owned())
                        .on_click(cx.listener(|this, event, _, _| {
                            this.clicks.push(matches!(
                                event,
                                gpui::ClickEvent::Keyboard(_)
                            ));
                        })),
                )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        disabled: bool,
        focusable_when_disabled: bool,
    ) -> (Entity<ButtonHarness>, &mut VisualTestContext) {
        cx.add_window_view(move |_window, _cx| ButtonHarness {
            disabled,
            focusable_when_disabled,
            clicks: Vec::new(),
            parent_clicks: 0,
        })
    }

    fn bounds(cx: &mut VisualTestContext) -> Bounds<Pixels> {
        cx.debug_bounds(BUTTON)
            .expect("base button should be rendered")
    }

    fn state(
        view: &Entity<ButtonHarness>,
        cx: &VisualTestContext,
    ) -> (Vec<bool>, usize) {
        cx.read_entity(view, |view, _cx| (view.clicks.clone(), view.parent_clicks))
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
    fn pointer_enter_and_space_activate_once_each(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, false, false);
        let button = bounds(cx);
        let inside = point(button.left() + px(10.), button.top() + px(10.));

        cx.simulate_click(inside, Modifiers::none());
        activate_key(cx, "enter");
        activate_key(cx, "space");

        assert_eq!(state(&view, cx).0, vec![false, true, true]);
    }

    #[gpui::test]
    fn disabled_button_is_inert_and_obeys_its_focus_policy(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, true, false);
        let button = bounds(cx);
        let inside = point(button.left() + px(10.), button.top() + px(10.));

        assert_eq!(button.size.width, px(80.));
        cx.simulate_click(inside, Modifiers::none());
        cx.update(|window, cx| {
            window.focus_next(cx);
            assert!(window.focused(cx).is_none());
        });
        assert_eq!(state(&view, cx), (Vec::new(), 0));

        view.update(cx, |view, cx| {
            view.focusable_when_disabled = true;
            cx.notify();
        });
        bounds(cx);
        cx.update(|window, cx| {
            window.focus_next(cx);
            assert!(window.focused(cx).is_some());
        });

        activate_key(cx, "enter");
        activate_key(cx, "space");
        cx.simulate_click(inside, Modifiers::none());

        assert_eq!(state(&view, cx), (Vec::new(), 0));
    }

    #[gpui::test]
    fn accessibility_exposes_role_label_and_available_action(
        cx: &mut TestAppContext,
    ) {
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
                        let mut info = |button: BaseButton| {
                            let mut node =
                                accesskit::Node::new(accesskit::Role::Button);
                            button
                                .render(window, cx)
                                .into_element()
                                .write_a11y_info(&mut node);
                            node
                        };

                        let enabled = info(
                            BaseButton::new("enabled")
                                .aria_label("Save")
                                .on_click(|_, _, _| {}),
                        );
                        let disabled = info(
                            BaseButton::new("disabled")
                                .disabled(true)
                                .focusable_when_disabled(true)
                                .aria_label("Save")
                                .on_click(|_, _, _| {}),
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

        assert_eq!(enabled.role(), accesskit::Role::Button);
        assert_eq!(enabled.label(), Some("Save"));
        assert!(enabled.supports_action(accesskit::Action::Click));

        assert_eq!(disabled.role(), accesskit::Role::Button);
        assert_eq!(disabled.label(), Some("Save"));
        assert!(!disabled.supports_action(accesskit::Action::Click));
    }
}
