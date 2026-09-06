#![cfg(feature = "test-support")]

use gpui::{
    AppContext, Bounds, Context, Entity, InteractiveElement, IntoElement, Modifiers,
    MouseButton, ParentElement, Pixels, Render, Styled, TestAppContext,
    VisualTestContext, Window, div, point, px,
};
use switch::Switch;
use tesserae_theme::{ThemeConfig, ThemeSet, ThemeSetKind};

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
fn click_and_small_movement_toggle_once(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, false, false);
    let bounds = host_bounds(cx);
    let inside = point(bounds.left() + px(5.), bounds.top() + px(5.));

    cx.simulate_click(inside, Modifiers::none());
    assert_eq!(state(&view, cx), (true, vec![true]));

    let small_move = point(inside.x + px(5.), inside.y);
    cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(small_move, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(small_move, MouseButton::Left, Modifiers::none());
    assert_eq!(state(&view, cx), (false, vec![true, false]));

    let outside_below = point(inside.x, bounds.bottom() + px(20.));
    cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(outside_below, MouseButton::Left, Modifiers::none());
    assert_eq!(state(&view, cx), (false, vec![true, false]));
}

#[gpui::test]
fn dragging_selects_a_value_and_tracks_outside(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, false, false);
    let bounds = host_bounds(cx);
    let inside = point(bounds.left() + px(20.), bounds.top() + px(10.));
    let outside_right = point(bounds.right() + px(20.), inside.y);

    cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(outside_right, MouseButton::Left, Modifiers::none());
    assert_eq!(state(&view, cx), (false, Vec::new()));

    cx.simulate_mouse_up(outside_right, MouseButton::Left, Modifiers::none());
    assert_eq!(state(&view, cx), (true, vec![true]));

    let outside_left = point(bounds.left() - px(20.), inside.y);
    cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(outside_left, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(outside_left, MouseButton::Left, Modifiers::none());
    assert_eq!(state(&view, cx), (false, vec![true, false]));

    cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(outside_left, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(outside_left, MouseButton::Left, Modifiers::none());
    assert_eq!(state(&view, cx), (false, vec![true, false]));
}

#[gpui::test]
fn disabled_switch_ignores_clicks_and_drags(cx: &mut TestAppContext) {
    let (view, cx) = setup(cx, false, true);
    let bounds = host_bounds(cx);
    let inside = point(bounds.left() + px(5.), bounds.top() + px(5.));
    let outside_right = point(bounds.right() + px(20.), inside.y);

    cx.simulate_click(inside, Modifiers::none());
    cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(outside_right, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(outside_right, MouseButton::Left, Modifiers::none());

    assert_eq!(state(&view, cx), (false, Vec::new()));

    view.update(cx, |view, cx| {
        view.disabled = false;
        cx.notify();
    });
    cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(outside_right, MouseButton::Left, Modifiers::none());

    view.update(cx, |view, cx| {
        view.disabled = true;
        cx.notify();
    });
    cx.simulate_mouse_up(outside_right, MouseButton::Left, Modifiers::none());
    assert_eq!(state(&view, cx), (false, Vec::new()));

    view.update(cx, |view, cx| {
        view.disabled = false;
        cx.notify();
    });
    cx.simulate_click(inside, Modifiers::none());
    assert_eq!(state(&view, cx), (true, vec![true]));
}
