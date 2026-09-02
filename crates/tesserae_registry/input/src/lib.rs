use gpui::{
    DurationWithEasing, ElementId, FontWeight, InteractiveElement, IntoElement,
    Lerp, Pixels, Rems, RenderOnce, StyleRefinement, Styled, Window, class,
    ease_in_out, millis, px, relative,
};
use gpui_elements::editable_text::{EditableTextElement, text_input};
use palette::{IntoColor, Oklaba, WithAlpha};
use tesserae_utils::{StyledElement, WindowUtils, focus_handle, kinds};

use tesserae_theme::{Theme, ThemeFgKind};

#[derive(IntoElement)]
pub struct Input {
    id: ElementId,
    size: InputSizeKind,
    variant: InputVariantKind,
    style: StyleRefinement,
}

impl Input {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            size: InputSizeKind::default(),
            variant: InputVariantKind::default(),
            style: StyleRefinement::default(),
        }
    }

    pub fn size(mut self, kind: InputSizeKind) -> Self {
        self.size = kind;
        self
    }

    pub fn size_md(self) -> Self {
        self.size(InputSizeKind::Md)
    }

    pub fn size_lg(self) -> Self {
        self.size(InputSizeKind::Lg)
    }

    pub fn variant(mut self, kind: InputVariantKind) -> Self {
        self.variant = kind;
        self
    }

    pub fn variant_primary(self) -> Self {
        self.variant(InputVariantKind::Primary)
    }

    pub fn variant_secondary(self) -> Self {
        self.variant(InputVariantKind::Secondary)
    }

    pub fn variant_tertiary(self) -> Self {
        self.variant(InputVariantKind::Tertiary)
    }

    pub fn variant_quaternary(self) -> Self {
        self.variant(InputVariantKind::Quaternary)
    }
}

impl RenderOnce for Input {
    fn render(
        self,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> impl gpui::IntoElement {
        let focus_handle = focus_handle(self.id.clone(), window, cx);

        let theme = Theme::read_global(cx);

        text_input(self.id.clone())
            .track_focus(&focus_handle)
            .placeholder("Type here...")
            .caret_blink_interval_500ms()
            .caret_h(relative(0.75))
            .selection_color(theme.accent_primary.with_alpha(0.3).into_color())
            .rounded_smoothing_1()
            .items_center()
            .flex()
            .line_height(theme.line_height)
            .font_family("Geist")
            .font_weight(FontWeight::NORMAL)
            .apply_kind(self.size, (window, theme))
            .apply_kind(self.variant, theme)
            .transitions(|transitions| {
                transitions.bg(millis(200).with_easing(ease_in_out))
            })
            .refine(self.style)
    }
}

impl Styled for Input {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

fn size_kind<E: Styled>(
    this: E,
    window: &Window,
    theme: &Theme,
    height: Rems,
    text_size: Rems,
    icon_size: Rems,
    radius: Rems,
    spacing: Pixels,
) -> E {
    this.rounded(radius)
        .gap(spacing)
        .px(spacing)
        .py(window.padding_for_height(height, text_size, theme.line_height))
        .text_size(text_size)
        .select_children(class("icon"), |refinement| refinement.size(icon_size))
}

kinds!(pub InputSizeKind<_, (&Window, &Theme)> {
    #[default]
    Md (this, (window, theme)) => {
        size_kind(
            this,
            window,
            theme,
            theme.size_2xl,
            theme.text_size_sm,
            theme.size_xs,
            theme.radii_lg,
            px(10.),
        )
    },

    Lg (this, (window, theme)) => {
        size_kind(
            this,
            window,
            theme,
            theme.size_3xl,
            theme.text_size_sm,
            theme.size_xs,
            theme.radii_lg,
            px(10.),
        )
    },
});

fn variant_kind(
    this: EditableTextElement,
    theme: &Theme,
    background: Oklaba,
    ring: Oklaba,
) -> EditableTextElement {
    this.bg(background)
        .text_color(theme.fg_for_bg(ThemeFgKind::Primary, background))
        .placeholder_color(theme.fg_for_bg(ThemeFgKind::Secondary, background))
        .inset_ring_1()
        .inset_ring_color(ring)
}

kinds!(pub InputVariantKind<EditableTextElement, &Theme> {
    Primary (this, theme) => {
        variant_kind(
            this,
            theme,
            theme.bg_primary,
            theme.bg_secondary.lerp(&theme.bg_tertiary, 0.5),
        )
    },

    Secondary (this, theme) => {
        variant_kind(
            this,
            theme,
            theme.bg_secondary,
            theme.bg_tertiary.lerp(&theme.bg_quaternary, 0.5),
        )
    },

    #[default]
    Tertiary (this, theme) => {
        variant_kind(
            this,
            theme,
            theme.bg_tertiary,
            theme.bg_quaternary.lerp(&theme.bg_quinary, 0.5),
        )
    },

    Quaternary (this, theme) => {
        variant_kind(
            this,
            theme,
            theme.bg_quaternary,
            theme.bg_quinary.lerp(&theme.bg_senary, 0.5),
        )
    },
});
