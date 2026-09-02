use gpui::{
    AnyElement, DurationWithEasing, ElementId, FontWeight, InteractiveElement,
    IntoElement, Lerp, ParentElement, RenderOnce, StyleRefinement, Styled, Window,
    div, ease_in_out, millis, px,
};
use tesserae_utils::{StyledElement, WindowUtils, focus_handle, kinds};

use smallvec::SmallVec;
use tesserae_theme::{Theme, ThemeFgKind};

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    size: ButtonSizeKind,
    variant: ButtonVariantKind,
    children: SmallVec<[AnyElement; 2]>,
    style: StyleRefinement,
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
            size: ButtonSizeKind::default(),
            variant: ButtonVariantKind::default(),
            children: SmallVec::new(),
            style: StyleRefinement::default(),
        }
    }

    pub fn size(mut self, kind: ButtonSizeKind) -> Self {
        self.size = kind;
        self
    }

    pub fn variant(mut self, kind: ButtonVariantKind) -> Self {
        self.variant = kind;
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
            .items_center()
            .justify_center()
            .flex()
            .line_height(theme.line_height)
            .font_family("Geist")
            .font_weight(FontWeight::MEDIUM)
            .apply_kind(self.size, (window, theme))
            .apply_kind(self.variant, theme)
            .children(self.children)
            .transitions(|transitions| {
                transitions.bg(millis(200).with_easing(ease_in_out))
            })
            .refine(self.style)
    }
}

kinds!(pub ButtonSizeKind<(&Window, &Theme)> {
    Xs (this, (window, theme)) => {
        this
            .rounded(theme.radii_md)
            .px(px(8.))
            .py(window.padding_for_height(
                theme.size_sm,
                theme.text_size_xs,
                theme.line_height
            ) - px(1.))
            .text_size(theme.text_size_xs)
    },

    Sm (this, (window, theme)) => {
        this
            .rounded(theme.radii_md)
            .px(px(8.))
            .py(window.padding_for_height(
                theme.size_md,
                theme.text_size_xs,
                theme.line_height
            ) - px(1.))
            .text_size(theme.text_size_xs)
    },

    #[default]
    Md (this, (window, theme)) => {
        this
            .rounded(theme.radii_lg)
            .px(px(10.))
            .py(window.padding_for_height(
                theme.size_lg,
                theme.text_size_sm,
                theme.line_height
            ) - px(1.))
            .text_size(theme.text_size_sm)
    },

    Lg (this, (window, theme)) => {
        this
            .rounded(theme.radii_lg)
            .px(px(10.))
            .py(window.padding_for_height(
                theme.size_xl,
                theme.text_size_sm,
                theme.line_height
            ) - px(1.))
            .text_size(theme.text_size_sm)
    }
});

kinds!(pub ButtonVariantKind<&Theme> {
    #[default]
    Primary (this, theme) => {
        this
            .bg(theme.accent_primary)
            .text_color(theme.fg_for_bg(ThemeFgKind::Primary, theme.accent_primary))
            .hover(|styles| styles
                .bg(theme.hover_color(theme.accent_primary))
            )
            .active(|styles| styles
                .bg(theme.active_color(theme.accent_primary))
            )
    },

    Secondary (this, theme) => {
        this
            .bg(theme.accent_secondary)
            .text_color(theme.fg_for_bg(ThemeFgKind::Primary, theme.accent_secondary))
            .hover(|styles| styles
                .bg(theme.hover_color(theme.accent_secondary))
            )
            .active(|styles| styles
                .bg(theme.active_color(theme.accent_secondary))
            )
    },

    Outline (this, theme) => {
        this
            .bg(theme.bg_secondary)
            .border_color(theme.bg_tertiary.lerp(&theme.bg_quaternary, 0.5))
            .border_1()
            .text_color(theme.fg_for_bg(ThemeFgKind::Primary, theme.bg_secondary))
            .hover(|styles| styles
                .bg(theme.hover_color(theme.bg_secondary))
            )
            .active(|styles| styles
                .bg(theme.active_color(theme.bg_secondary))
            )
    }
});
