use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, DurationWithEasing, ElementId, FontWeight,
    InteractiveElement, IntoElement, Lerp, MouseButton, ParentElement, Rems,
    RenderOnce, StatefulInteractiveElement, StyleRefinement, Styled, Window, div,
    ease_in_out, linear_color_stop, linear_gradient, millis, prelude::FluentBuilder,
    selectors::class,
};
use palette::{Oklaba, WithAlpha};
use tesserae_utils::{
    PerceptualColor, StyledElement, WindowUtils, kinds, use_focus_handle,
};

use smallvec::SmallVec;
use tesserae_theme::{Theme, ThemeFgKind};

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    size: ButtonSizeKind,
    variant: ButtonVariantKind,
    disabled: bool,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
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
            disabled: false,
            on_click: None,
            children: SmallVec::new(),
            style: StyleRefinement::default(),
        }
    }

    pub fn size(mut self, kind: ButtonSizeKind) -> Self {
        self.size = kind;
        self
    }

    pub fn xs(self) -> Self {
        self.size(ButtonSizeKind::Xs)
    }

    pub fn xs_icon(self) -> Self {
        self.size(ButtonSizeKind::XsIcon)
    }

    pub fn sm(self) -> Self {
        self.size(ButtonSizeKind::Sm)
    }

    pub fn sm_icon(self) -> Self {
        self.size(ButtonSizeKind::SmIcon)
    }

    pub fn md(self) -> Self {
        self.size(ButtonSizeKind::Md)
    }

    pub fn md_icon(self) -> Self {
        self.size(ButtonSizeKind::MdIcon)
    }

    pub fn lg(self) -> Self {
        self.size(ButtonSizeKind::Lg)
    }

    pub fn lg_icon(self) -> Self {
        self.size(ButtonSizeKind::LgIcon)
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

    pub fn on_click(
        mut self,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }
}

impl RenderOnce for Button {
    fn render(
        self,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> impl gpui::IntoElement {
        let focus_handle = use_focus_handle(self.id.clone(), window, cx, None);
        let theme = Theme::read_global(cx);
        let disabled = self.disabled;

        div()
            .id(self.id)
            .rounded_smoothing_1()
            .items_center()
            .justify_center()
            .flex()
            .line_height(theme.line_height)
            .font_family("Geist")
            .font_weight(FontWeight::MEDIUM)
            .apply_kind(self.size, (window, theme))
            .apply_kind(self.variant, (theme, disabled))
            .transitions(|transitions| {
                transitions.bg(millis(200).with_easing(ease_in_out))
            })
            .when_else(
                !disabled,
                |this| {
                    this.track_focus(&focus_handle).cursor_pointer().when_some(
                        self.on_click,
                        |this, on_click| {
                            this.on_click(move |event, window, cx| {
                                on_click(event, window, cx);
                            })
                        },
                    )
                },
                |this| {
                    this.cursor_not_allowed().opacity(0.48).on_mouse_down(
                        MouseButton::Left,
                        |_, _, cx| {
                            cx.stop_propagation();
                        },
                    )
                },
            )
            .children(self.children)
            .refine_style(&self.style)
    }
}

impl Styled for Button {
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
    spacing: Rems,
) -> E {
    this.rounded(radius)
        .gap(spacing)
        .px(spacing)
        .py(window.padding_for_height(height, text_size, theme.line_height))
        .text_size(text_size)
        .select_children(class("icon"), |refinement| refinement.size(icon_size))
}

fn icon_size_kind<E: Styled>(
    this: E,
    theme: &Theme,
    size: Rems,
    text_size: Rems,
    icon_size: Rems,
    radius: Rems,
) -> E {
    this.rounded(radius)
        .size(size)
        .line_height(theme.line_height)
        .text_size(text_size)
        .select_children(class("icon"), |refinement| refinement.size(icon_size))
}

kinds!(pub ButtonSizeKind<_, (&Window, &Theme)> {
    Xs (this, (window, theme)) => {
        size_kind(
            this,
            window,
            theme,
            theme.size_3xl,
            theme.text_size_xs,
            theme.size_xs,
            theme.radii_md,
            theme.padding_lg,
        )
    },

    XsIcon (this, (_window, theme)) => {
        icon_size_kind(
            this,
            theme,
            theme.size_3xl,
            theme.text_size_xs,
            theme.size_xs,
            theme.radii_md,
        )
    },

    Sm (this, (window, theme)) => {
        size_kind(
            this,
            window,
            theme,
            theme.size_4xl,
            theme.text_size_xs,
            theme.size_sm,
            theme.radii_md,
            theme.padding_xl,
        )
    },

    SmIcon (this, (_window, theme)) => {
        icon_size_kind(
            this,
            theme,
            theme.size_4xl,
            theme.text_size_xs,
            theme.size_sm,
            theme.radii_md,
        )
    },

    #[default]
    Md (this, (window, theme)) => {
        size_kind(
            this,
            window,
            theme,
            theme.size_5xl,
            theme.text_size_sm,
            theme.size_md,
            theme.radii_lg,
            theme.padding_xl,
        )
    },

    MdIcon (this, (_window, theme)) => {
        icon_size_kind(
            this,
            theme,
            theme.size_5xl,
            theme.text_size_sm,
            theme.size_md,
            theme.radii_lg,
        )
    },

    Lg (this, (window, theme)) => {
        size_kind(
            this,
            window,
            theme,
            theme.size_6xl,
            theme.text_size_sm,
            theme.size_md,
            theme.radii_lg,
            theme.padding_xl,
        )
    },

    LgIcon (this, (_window, theme)) => {
        icon_size_kind(
            this,
            theme,
            theme.size_6xl,
            theme.text_size_sm,
            theme.size_md,
            theme.radii_lg,
        )
    },
});

fn fill_button_variant_kind<E>(
    this: E,
    theme: &Theme,
    bg_color: Oklaba,
    disabled: bool,
) -> E
where
    E: Styled + StatefulInteractiveElement + FluentBuilder,
{
    let fg_color = theme.fg_for_bg(ThemeFgKind::Primary, bg_color);

    this.bg(bg_color)
        .text_color(fg_color)
        .when(!disabled, |this| {
            this.hover(|styles| styles.bg(theme.hover_feedback(bg_color)))
                .active(|styles| styles.bg(theme.active_feedback(bg_color)))
        })
        .inset_ring_1()
        .inset_ring_color(linear_gradient(
            180.,
            linear_color_stop(theme.fg_primary.perceptual_alpha(0.08), 0.),
            linear_color_stop(theme.fg_primary.with_alpha(0.), 0.8),
        ))
        .select_children(class("icon"), |refinement| refinement.text_color(fg_color))
}

kinds!(pub ButtonVariantKind<Styled + StatefulInteractiveElement + FluentBuilder, (&Theme, bool)> {
    #[default]
    Primary (this, (theme, disabled)) => {
        fill_button_variant_kind(this, theme, theme.accent_primary, disabled)
    },

    Secondary (this, (theme, disabled)) => {
        fill_button_variant_kind(this, theme, theme.accent_secondary, disabled)
    },

    Outline (this, (theme, disabled)) => {
        let fg_color =
            theme.fg_for_bg(ThemeFgKind::Primary, theme.bg_secondary);

        this
            .bg(theme.bg_secondary)
            .inset_ring_1()
            .inset_ring_color(theme.bg_tertiary.lerp(&theme.bg_quaternary, 0.5))
            .text_color(fg_color)
            .when(!disabled, |this| {
                this.hover(|styles| styles
                    .bg(theme.hover_feedback(theme.bg_secondary))
                )
                .active(|styles| styles
                    .bg(theme.active_feedback(theme.bg_secondary))
                )
            })
            .select_children(class("icon"), |refinement| refinement.text_color(fg_color))
    }
});
