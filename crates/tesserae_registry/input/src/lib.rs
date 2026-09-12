use focus_ring::FocusRing;
use gpui::{
    DurationWithEasing, ElementId, Focusable, FontWeight, InteractiveElement,
    IntoElement, Lerp, ParentElement, Rems, RenderOnce, StyleRefinement, Styled,
    Window, div, ease_in_out, millis, prelude::FluentBuilder, relative,
    selectors::class,
};
use gpui_elements::editable_text::{EditableTextState, text_input};
use palette::{IntoColor, Oklaba, WithAlpha};
use tesserae_macros::Styles;
use tesserae_theme::Theme;
use tesserae_utils::{PerceptualColor, StyledElement, WindowUtils};

#[derive(IntoElement)]
pub struct Input {
    element_id: ElementId,
    size: InputSizeKind,
    variant: InputVariantKind,
    style: StyleRefinement,
}

impl Input {
    pub fn new(element_id: impl Into<ElementId>) -> Self {
        Self {
            element_id: element_id.into(),
            size: InputSizeKind::default(),
            variant: InputVariantKind::default(),
            style: StyleRefinement::default(),
        }
    }

    pub fn size(mut self, kind: InputSizeKind) -> Self {
        self.size = kind;

        self
    }

    pub fn medium(self) -> Self {
        self.size(InputSizeKind::Medium)
    }

    pub fn large(self) -> Self {
        self.size(InputSizeKind::Large)
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
        let input_state = EditableTextState::use_keyed(
            (self.element_id.clone(), "state"),
            window,
            cx,
        );

        let focus_handle = input_state.focus_handle(cx).tab_stop(true);

        let theme = Theme::read_global(cx);
        let is_focused = focus_handle.is_focused(window);
        let variant_ring_color = self.variant.colors(theme).1;

        div()
            .id(self.element_id.clone())
            .rounded_smoothing_1()
            .inset_ring_1()
            .flex()
            .justify_center()
            .refine_styles_with_enum(self.size, (window, theme))
            .refine_styles_with_enum(self.variant, theme)
            .when(!is_focused, |this| {
                this.hover(|this| {
                    this.inset_ring_color(theme.hover_feedback(variant_ring_color))
                })
            })
            .when(is_focused, |this| {
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
                (self.element_id.clone(), "focus_ring"),
                focus_handle,
            ))
            .child(
                text_input((self.element_id, "input"))
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
                    .refine_style(&self.style),
            )
    }
}

impl Styled for Input {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

fn size_styles(
    refinement: StyleRefinement,
    window: &Window,
    theme: &Theme,
    height: Rems,
    text_size: Rems,
    icon_size: Rems,
    radius: Rems,
    spacing: Rems,
) -> StyleRefinement {
    refinement
        .rounded(radius)
        .gap(spacing)
        .px(spacing)
        .py(window.padding_for_height(height, text_size, theme.line_height))
        .text_size(text_size)
        .select_children(class("icon"), |refinement| refinement.size(icon_size))
        .select_children(class("focus_ring"), |refinement| {
            refinement.rounded(radius)
        })
}

#[derive(Clone, Copy, Default, Styles)]
#[styles_data(&Window, &Theme)]
pub enum InputSizeKind {
    #[default]
    #[styles(|refinement, (window, theme)| {
        size_styles(
            refinement,
            window,
            theme,
            theme.size_5xl,
            theme.text_size_sm,
            theme.size_md,
            theme.radii_lg,
            theme.padding_xl,
        )
    })]
    Medium,

    #[styles(|refinement, (window, theme)| {
        size_styles(
            refinement,
            window,
            theme,
            theme.size_6xl,
            theme.text_size_sm,
            theme.size_md,
            theme.radii_lg,
            theme.padding_xl,
        )
    })]
    Large,
}

fn variant_styles(
    refinement: StyleRefinement,
    background: Oklaba,
    ring_color: Oklaba,
) -> StyleRefinement {
    refinement.bg(background).inset_ring_color(ring_color)
}

#[derive(Clone, Copy, Default, Styles)]
#[styles_data(&Theme)]
pub enum InputVariantKind {
    #[styles(|refinement, theme| {
        let (background, ring_color) =
            InputVariantKind::Primary.colors(theme);

        variant_styles(refinement, background, ring_color)
    })]
    Primary,

    #[styles(|refinement, theme| {
        let (background, ring_color) =
            InputVariantKind::Secondary.colors(theme);

        variant_styles(refinement, background, ring_color)
    })]
    Secondary,

    #[default]
    #[styles(|refinement, theme| {
        let (background, ring_color) =
            InputVariantKind::Tertiary.colors(theme);

        variant_styles(refinement, background, ring_color)
    })]
    Tertiary,

    #[styles(|refinement, theme| {
        let (background, ring_color) =
            InputVariantKind::Quaternary.colors(theme);

        variant_styles(refinement, background, ring_color)
    })]
    Quaternary,
}

impl InputVariantKind {
    fn colors(self, theme: &Theme) -> (Oklaba, Oklaba) {
        match self {
            Self::Primary => (
                theme.bg_primary,
                theme.bg_secondary.lerp(&theme.bg_tertiary, 0.5),
            ),
            Self::Secondary => (
                theme.bg_secondary,
                theme.bg_tertiary.lerp(&theme.bg_quaternary, 0.5),
            ),
            Self::Tertiary => (
                theme.bg_tertiary,
                theme.bg_quaternary.lerp(&theme.bg_quinary, 0.5),
            ),
            Self::Quaternary => (
                theme.bg_quaternary,
                theme.bg_quinary.lerp(&theme.bg_senary, 0.5),
            ),
        }
    }
}
