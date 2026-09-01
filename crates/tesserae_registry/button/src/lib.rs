use gpui::{
    AnyElement, DurationWithEasing, ElementId, FontWeight, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, Styled, Window, div, ease_in_out,
    millis, px,
};
use tesserae_utils::{StyledElement, WindowUtils, focus_handle, variants};

use smallvec::SmallVec;
use tesserae_theme::Theme;

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    variant: ButtonVariant,
    children: SmallVec<[AnyElement; 2]>,
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: ButtonVariant::default(),
            children: SmallVec::new(),
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl RenderOnce for Button {
    fn render(
        self,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> impl gpui::IntoElement {
        let focus_handle = focus_handle(self.id.clone(), window, cx);

        let theme = Theme::read_global(cx);

        div()
            .id(self.id)
            .track_focus(&focus_handle)
            .cursor_pointer()
            .rounded_smoothing_1()
            .text_color(theme.fg_primary)
            .border_1()
            .items_center()
            .justify_center()
            .flex()
            .line_height(theme.line_height)
            .font_family("Geist")
            .font_weight(FontWeight::MEDIUM)
            .apply_variant(self.variant, (window, theme))
            .children(self.children)
            .transitions(|transitions| {
                transitions.bg(millis(200).with_easing(ease_in_out))
            })
    }
}

variants!(pub ButtonVariant<(&Window, &Theme)> {
    Xs (this, (window, theme)) => {
        this
            .bg(theme.bg_secondary)
            .border_color(theme.bg_tertiary)
            .border_1()
            .rounded(theme.radii_md)
            .px(px(8.))
            .py(window.padding_for_height(
                theme.size_sm,
                theme.text_size_xs,
                theme.line_height
            ) - px(1.))
            .text_size(theme.text_size_xs)
            .hover(|styles| styles
                .bg(theme.hover_color(theme.bg_secondary))
            )
            .active(|styles| styles
                .bg(theme.active_color(theme.bg_secondary))
            )
    },

    Sm (this, (window, theme)) => {
        this
            .bg(theme.bg_secondary)
            .border_color(theme.bg_tertiary)
            .border_1()
            .rounded(theme.radii_md)
            .px(px(8.))
            .py(window.padding_for_height(
                theme.size_md,
                theme.text_size_xs,
                theme.line_height
            ) - px(1.))
            .text_size(theme.text_size_xs)
            .hover(|styles| styles
                .bg(theme.hover_color(theme.bg_secondary))
            )
            .active(|styles| styles
                .bg(theme.active_color(theme.bg_secondary))
            )
    },

    #[default]
    Md (this, (window, theme)) => {
        this
            .bg(theme.bg_secondary)
            .border_color(theme.bg_tertiary)
            .border_1()
            .rounded(theme.radii_lg)
            .px(px(10.))
            .py(window.padding_for_height(
                theme.size_lg,
                theme.text_size_sm,
                theme.line_height
            ) - px(1.))
            .text_size(theme.text_size_sm)
            .hover(|styles| styles
                .bg(theme.hover_color(theme.bg_secondary))
            )
            .active(|styles| styles
                .bg(theme.active_color(theme.bg_secondary))
            )
    },

    Lg (this, (window, theme)) => {
        this
            .bg(theme.bg_secondary)
            .border_color(theme.bg_tertiary)
            .border_1()
            .rounded(theme.radii_lg)
            .px(px(10.))
            .py(window.padding_for_height(
                theme.size_xl,
                theme.text_size_sm,
                theme.line_height
            ) - px(1.))
            .text_size(theme.text_size_sm)
            .hover(|styles| styles
                .bg(theme.hover_color(theme.bg_secondary))
            )
            .active(|styles| styles
                .bg(theme.active_color(theme.bg_secondary))
            )
    }
});
