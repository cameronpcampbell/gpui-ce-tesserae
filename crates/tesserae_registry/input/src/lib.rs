use focus_ring::FocusRing;
use gpui::{
    DurationWithEasing, ElementId, Focusable, FontWeight, InteractiveElement,
    IntoElement, Lerp, ParentElement, Pixels, Rems, RenderOnce, StyleRefinement,
    Styled, Window, div, ease_in_out, millis,
    prelude::FluentBuilder,
    px, relative,
    selectors::{class, tag},
};
use gpui_elements::editable_text::{EditableTextState, text_input};
use palette::{IntoColor, Oklaba, WithAlpha};
use tesserae_utils::{StyledElement, WindowUtils, kinds};

use tesserae_theme::Theme;

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
        let input_state =
            EditableTextState::use_keyed((self.id.clone(), "state"), window, cx);

        let focus_handle = input_state.focus_handle(cx).tab_stop(true);

        let theme = Theme::read_global(cx);

        div()
            .rounded_smoothing_1()
            .inset_ring_1()
            .flex()
            .justify_center()
            .apply_kind(self.size, (window, theme))
            .apply_kind(self.variant, theme)
            .when(focus_handle.is_focused(window), |this| {
                this.inset_ring_color(theme.accent_primary)
            })
            .child(
                text_input(self.id.clone())
                    .state(input_state.downgrade())
                    .placeholder("Type here...")
                    .caret_blink_interval_500ms()
                    .caret_h(relative(0.75))
                    .text_color(theme.fg_primary)
                    .placeholder_color(theme.fg_secondary)
                    .selection_color(
                        theme.accent_primary.with_alpha(0.3).into_color(),
                    )
                    .line_height(theme.line_height)
                    .font_family("Geist")
                    .font_weight(FontWeight::NORMAL)
                    .transitions(|transitions| {
                        transitions
                            .bg(millis(200).with_easing(ease_in_out))
                            .inset_ring_color(millis(120).with_easing(ease_in_out))
                    })
                    .refine(self.style),
            )
            .child(FocusRing::new((self.id, "focus_ring"), focus_handle))
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
        .select_descendants(tag::<FocusRing>(), |refinement| {
            refinement.rounded(radius)
        })
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

fn variant_kind<E: Styled>(this: E, bg: Oklaba, ring_color: Oklaba) -> E {
    this.bg(bg).inset_ring_color(ring_color)
}

kinds!(pub InputVariantKind<_, &Theme> {
    Primary (this, theme) => {
        variant_kind(
            this,
            theme.bg_primary,
            theme.bg_secondary.lerp(&theme.bg_tertiary, 0.5),
        )
    },

    Secondary (this, theme) => {
        variant_kind(
            this,
            theme.bg_secondary,
            theme.bg_tertiary.lerp(&theme.bg_quaternary, 0.5),
        )
    },

    #[default]
    Tertiary (this, theme) => {
        variant_kind(
            this,
            theme.bg_tertiary,
            theme.bg_quaternary.lerp(&theme.bg_quinary, 0.5),
        )
    },

    Quaternary (this, theme) => {
        variant_kind(
            this,
            theme.bg_quaternary,
            theme.bg_quinary.lerp(&theme.bg_senary, 0.5),
        )
    },
});
