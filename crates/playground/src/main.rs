use button::ButtonVariant;
use gpui::{
    App, Bounds, KeyBinding, WindowOptions, actions, div, prelude::*, px, size,
};
use tesserae_theme::{Theme, ThemeConfig, ThemeSet, ThemeSetKind};

mod assets;
use assets::Assets;

struct Root;

impl Render for Root {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = Theme::read_global(cx);

        div()
            .id("base")
            .tab_group()
            .tab_stop(false)
            .tab_index(0)
            .bg(theme.bg_primary)
            .flex()
            .justify_center()
            .items_center()
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(10.))
                    .child(
                        button::Button::new("button_xs")
                            .variant(ButtonVariant::Xs)
                            .child("Extra Small"),
                    )
                    .child(
                        button::Button::new("button_sm")
                            .variant(ButtonVariant::Sm)
                            .child("Small"),
                    )
                    .child(
                        button::Button::new("button_md")
                            .variant(ButtonVariant::Md)
                            .child("Medium"),
                    )
                    .child(
                        button::Button::new("button_lg")
                            .variant(ButtonVariant::Lg)
                            .child("Large"),
                    ),
            )
    }
}

fn main() {
    gpui_platform::application().with_assets(Assets).run(|cx| {
        Assets::init(cx).ok();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(400.), px(400.)),
                    cx,
                ))),
                ..Default::default()
            },
            |_window, cx| {
                ThemeSet::set_global(cx, ThemeSet::generate(ThemeConfig::default()));

                let theme = ThemeSet::generate(ThemeConfig::default());

                println!("{:#?}", theme.dark);

                ThemeSetKind::set_global(cx, ThemeSetKind::Dark);

                cx.new(|_cx| Root)
            },
        )
        .ok();

        cx.activate(true);

        init_tab_indexing_actions(cx);
    });
}

actions!(window, [TabNext, TabPrev]);

fn init_tab_indexing_actions(cx: &mut App) {
    cx.on_action(move |_: &TabNext, cx| {
        cx.defer(move |cx| {
            let Some(window) = cx.active_window() else {
                return;
            };

            let _ = window.update(cx, move |_, window, cx| {
                window.focus_next(cx);
            });
        })
    });

    cx.on_action(move |_: &TabPrev, cx| {
        cx.defer(move |cx| {
            let Some(window) = cx.active_window() else {
                return;
            };

            let _ = window.update(cx, move |_, window, cx| {
                window.focus_prev(cx);
            });
        })
    });

    cx.bind_keys([KeyBinding::new("tab", TabNext, None)]);
    cx.bind_keys([KeyBinding::new("shift-tab", TabPrev, None)]);
}
