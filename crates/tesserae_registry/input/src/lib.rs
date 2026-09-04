use focus_ring::FocusRing;
use gpui::{
    DurationWithEasing, ElementId, Focusable, FontWeight, InteractiveElement,
    IntoElement, Lerp, ParentElement, Pixels, Rems, RenderOnce, StyleRefinement,
    Styled, Window, div, ease_in_out, millis, prelude::FluentBuilder, px, relative,
    selectors::class,
};
use gpui_elements::editable_text::{EditableTextState, text_input};
use palette::{IntoColor, Oklaba, WithAlpha};
use tesserae_utils::{PerceptualColor, StyledElement, WindowUtils, kinds};

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

    pub fn md(self) -> Self {
        self.size(InputSizeKind::Md)
    }

    pub fn lg(self) -> Self {
        self.size(InputSizeKind::Lg)
    }

    pub fn variant(mut self, kind: InputVariantKind) -> Self {
        self.variant = kind;
        self
    }

    pub fn primary(self) -> Self {
        self.variant(InputVariantKind::Primary)
    }

    pub fn secondary(self) -> Self {
        self.variant(InputVariantKind::Secondary)
    }

    pub fn tertiary(self) -> Self {
        self.variant(InputVariantKind::Tertiary)
    }

    pub fn quaternary(self) -> Self {
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
            .id(self.id.clone())
            .rounded_smoothing_1()
            .inset_ring_1()
            .flex()
            .justify_center()
            .apply_kind(self.size, (window, theme))
            .apply_kind(self.variant, (theme, focus_handle.is_focused(window)))
            .when(focus_handle.is_focused(window), |this| {
                let inset_ring_color =
                    theme.accent_primary.perceptual_brightness(0.5);

                this.inset_ring_color(inset_ring_color).hover(|this| {
                    this.inset_ring_color(theme.hover_feedback(inset_ring_color))
                })
            })
            .transitions(|transitions| {
                transitions.inset_ring_color(millis(120).with_easing(ease_in_out))
            })
            .child(FocusRing::new(
                (self.id.clone(), "focus_ring"),
                focus_handle,
            ))
            .child(
                text_input((self.id, "input"))
                    .class("input")
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
                    .refine(self.style),
            )
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
        .select_children(class("focus_ring"), |refinement| {
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

fn variant_kind<E: Styled + InteractiveElement + FluentBuilder>(
    this: E,
    theme: &Theme,
    is_focused: bool,
    bg: Oklaba,
    ring_color: Oklaba,
) -> E {
    this.bg(bg)
        .inset_ring_color(ring_color)
        .when(!is_focused, |this| {
            this.hover(|this| {
                this.inset_ring_color(theme.hover_feedback(ring_color))
            })
        })
}

kinds!(pub InputVariantKind<Styled + InteractiveElement + FluentBuilder, (&Theme, bool)> {
    Primary (this, (theme, is_focused)) => {
        variant_kind(
            this,
            theme,
            is_focused,
            theme.bg_primary,
            theme.bg_secondary.lerp(&theme.bg_tertiary, 0.5),
        )
    },

    Secondary (this, (theme, is_focused)) => {
        variant_kind(
            this,
            theme,
            is_focused,
            theme.bg_secondary,
            theme.bg_tertiary.lerp(&theme.bg_quaternary, 0.5),
        )
    },

    #[default]
    Tertiary (this, (theme, is_focused)) => {
        variant_kind(
            this,
            theme,
            is_focused,
            theme.bg_tertiary,
            theme.bg_quaternary.lerp(&theme.bg_quinary, 0.5),
        )
    },

    Quaternary (this, (theme, is_focused)) => {
        variant_kind(
            this,
            theme,
            is_focused,
            theme.bg_quaternary,
            theme.bg_quinary.lerp(&theme.bg_senary, 0.5),
        )
    },
});
