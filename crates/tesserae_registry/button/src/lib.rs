use std::rc::Rc;

use focus_ring::FocusRing;
use gpui::{
    AnyElement, App, ClickEvent, DurationWithEasing, ElementId, FontWeight,
    InteractiveElement, IntoElement, Lerp, ParentElement, Rems, RenderOnce,
    SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Window,
    ease_in_out, linear_color_stop, linear_gradient, millis, prelude::FluentBuilder,
    selectors::class,
};
use palette::{Oklaba, WithAlpha};
use tesserae_base::button::BaseButton;
use tesserae_macros::Styles;
use tesserae_theme::{Theme, ThemeFgKind};
use tesserae_utils::{
    PerceptualColor, StyledElement, WindowUtils, use_focus_handle,
};

#[derive(IntoElement)]
pub struct Button {
    element_id: ElementId,
    size: ButtonSizeKind,
    variant: ButtonVariantKind,
    disabled: bool,
    focusable_when_disabled: bool,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    aria_label: Option<SharedString>,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Button {
    pub fn new(element_id: impl Into<ElementId>) -> Self {
        Self {
            element_id: element_id.into(),
            size: ButtonSizeKind::default(),
            variant: ButtonVariantKind::default(),
            disabled: false,
            focusable_when_disabled: false,
            on_click: None,
            aria_label: None,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    pub fn size(mut self, kind: ButtonSizeKind) -> Self {
        self.size = kind;

        self
    }

    pub fn extra_small(self) -> Self {
        self.size(ButtonSizeKind::ExtraSmall)
    }

    pub fn extra_small_icon(self) -> Self {
        self.size(ButtonSizeKind::ExtraSmallIcon)
    }

    pub fn small(self) -> Self {
        self.size(ButtonSizeKind::Small)
    }

    pub fn small_icon(self) -> Self {
        self.size(ButtonSizeKind::SmallIcon)
    }

    pub fn medium(self) -> Self {
        self.size(ButtonSizeKind::Medium)
    }

    pub fn medium_icon(self) -> Self {
        self.size(ButtonSizeKind::MediumIcon)
    }

    pub fn large(self) -> Self {
        self.size(ButtonSizeKind::Large)
    }

    pub fn large_icon(self) -> Self {
        self.size(ButtonSizeKind::LargeIcon)
    }

    pub fn variant(mut self, kind: ButtonVariantKind) -> Self {
        self.variant = kind;

        self
    }

    pub fn primary(self) -> Self {
        self.variant(ButtonVariantKind::Primary)
    }

    pub fn secondary(self) -> Self {
        self.variant(ButtonVariantKind::Secondary)
    }

    pub fn outline(self) -> Self {
        self.variant(ButtonVariantKind::Outline)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;

        self
    }

    pub fn focusable_when_disabled(mut self, focusable: bool) -> Self {
        self.focusable_when_disabled = focusable;

        self
    }

    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());

        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));

        self
    }
}

impl RenderOnce for Button {
    fn render(
        self,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> impl gpui::IntoElement {
        let focus_handle =
            use_focus_handle(self.element_id.clone(), window, cx, None);
        let theme = Theme::read_global(cx);

        let disabled = self.disabled;
        let variant_background = self.variant.background(theme);

        BaseButton::new(self.element_id.clone())
            .disabled(disabled)
            .focusable_when_disabled(self.focusable_when_disabled)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_click, |this, on_click| {
                this.on_click(move |event, window, cx| {
                    on_click(event, window, cx);
                })
            })
            .rounded_smoothing_1()
            .items_center()
            .justify_center()
            .flex()
            .line_height(theme.line_height)
            .font_family("Geist")
            .font_weight(FontWeight::MEDIUM)
            .refine_styles_with_enum(self.size, (window, theme))
            .refine_styles_with_enum(self.variant, theme)
            .when(!disabled, |this| {
                this.hover(|styles| {
                    styles.bg(theme.hover_feedback(variant_background))
                })
                .active(|styles| {
                    styles.bg(theme.active_feedback(variant_background))
                })
            })
            .transitions(|transitions| {
                transitions.bg(millis(200).with_easing(ease_in_out))
            })
            .when_else(
                !disabled,
                |this| this.cursor_pointer(),
                |this| this.cursor_not_allowed().opacity(0.48),
            )
            .child(FocusRing::new(
                (self.element_id, "focus_ring"),
                focus_handle,
            ))
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for Button {
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
}

fn icon_size_styles(
    refinement: StyleRefinement,
    theme: &Theme,
    size: Rems,
    text_size: Rems,
    icon_size: Rems,
    radius: Rems,
) -> StyleRefinement {
    refinement
        .rounded(radius)
        .size(size)
        .line_height(theme.line_height)
        .text_size(text_size)
        .select_children(class("icon"), |refinement| refinement.size(icon_size))
}

#[derive(Clone, Copy, Default, Styles)]
#[styles_data(&Window, &Theme)]
pub enum ButtonSizeKind {
    #[styles(|refinement, (window, theme)| {
        size_styles(
            refinement,
            window,
            theme,
            theme.size_3xl,
            theme.text_size_xs,
            theme.size_xs,
            theme.radii_md,
            theme.padding_lg,
        )
    })]
    ExtraSmall,

    #[styles(|refinement, (_window, theme)| {
        icon_size_styles(
            refinement,
            theme,
            theme.size_3xl,
            theme.text_size_xs,
            theme.size_xs,
            theme.radii_md,
        )
    })]
    ExtraSmallIcon,

    #[styles(|refinement, (window, theme)| {
        size_styles(
            refinement,
            window,
            theme,
            theme.size_4xl,
            theme.text_size_xs,
            theme.size_sm,
            theme.radii_md,
            theme.padding_xl,
        )
    })]
    Small,

    #[styles(|refinement, (_window, theme)| {
        icon_size_styles(
            refinement,
            theme,
            theme.size_4xl,
            theme.text_size_xs,
            theme.size_sm,
            theme.radii_md,
        )
    })]
    SmallIcon,

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

    #[styles(|refinement, (_window, theme)| {
        icon_size_styles(
            refinement,
            theme,
            theme.size_5xl,
            theme.text_size_sm,
            theme.size_md,
            theme.radii_lg,
        )
    })]
    MediumIcon,

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

    #[styles(|refinement, (_window, theme)| {
        icon_size_styles(
            refinement,
            theme,
            theme.size_6xl,
            theme.text_size_sm,
            theme.size_md,
            theme.radii_lg,
        )
    })]
    LargeIcon,
}

fn fill_button_variant_styles(
    refinement: StyleRefinement,
    theme: &Theme,
    bg_color: Oklaba,
) -> StyleRefinement {
    let fg_color = theme.foreground_for_background(ThemeFgKind::Primary, bg_color);

    refinement
        .bg(bg_color)
        .text_color(fg_color)
        .inset_ring_1()
        .inset_ring_color(linear_gradient(
            180.,
            linear_color_stop(theme.fg_primary.perceptual_alpha(0.08), 0.),
            linear_color_stop(theme.fg_primary.with_alpha(0.), 0.8),
        ))
        .select_children(class("icon"), |refinement| refinement.text_color(fg_color))
}

#[derive(Clone, Copy, Default, Styles)]
#[styles_data(&Theme)]
pub enum ButtonVariantKind {
    #[default]
    #[styles(|refinement, theme| {
        fill_button_variant_styles(
            refinement,
            theme,
            ButtonVariantKind::Primary.background(theme),
        )
    })]
    Primary,

    #[styles(|refinement, theme| {
        fill_button_variant_styles(
            refinement,
            theme,
            ButtonVariantKind::Secondary.background(theme),
        )
    })]
    Secondary,

    #[styles(|refinement, theme| {
        let fg_color = theme
            .foreground_for_background(ThemeFgKind::Primary, theme.bg_secondary);

        refinement
            .bg(theme.bg_secondary)
            .inset_ring_1()
            .inset_ring_color(theme.bg_tertiary.lerp(&theme.bg_quaternary, 0.5))
            .text_color(fg_color)
            .select_children(class("icon"), |refinement| refinement.text_color(fg_color))
    })]
    Outline,
}

impl ButtonVariantKind {
    fn background(self, theme: &Theme) -> Oklaba {
        match self {
            Self::Primary => theme.accent_primary,
            Self::Secondary => theme.accent_secondary,
            Self::Outline => theme.bg_secondary,
        }
    }
}
