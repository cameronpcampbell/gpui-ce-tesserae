use gpui::{
    AnyElement, DurationWithEasing, ElementId, FontWeight, InteractiveElement,
    IntoElement, Lerp, ParentElement, Pixels, Rems, RenderOnce,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, class, div,
    ease_in_out, millis, px,
};
use palette::Oklaba;
use tesserae_utils::{StyledElement, WindowUtils, kinds, use_focus_handle};

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

    pub fn size_xs(self) -> Self {
        self.size(ButtonSizeKind::Xs)
    }

    pub fn size_xs_icon(self) -> Self {
        self.size(ButtonSizeKind::XsIcon)
    }

    pub fn size_sm(self) -> Self {
        self.size(ButtonSizeKind::Sm)
    }

    pub fn size_sm_icon(self) -> Self {
        self.size(ButtonSizeKind::SmIcon)
    }

    pub fn size_md(self) -> Self {
        self.size(ButtonSizeKind::Md)
    }

    pub fn size_md_icon(self) -> Self {
        self.size(ButtonSizeKind::MdIcon)
    }

    pub fn size_lg(self) -> Self {
        self.size(ButtonSizeKind::Lg)
    }

    pub fn size_lg_icon(self) -> Self {
        self.size(ButtonSizeKind::LgIcon)
    }

    pub fn variant(mut self, kind: ButtonVariantKind) -> Self {
        self.variant = kind;
        self
    }

    pub fn variant_primary(self) -> Self {
        self.variant(ButtonVariantKind::Primary)
    }

    pub fn variant_secondary(self) -> Self {
        self.variant(ButtonVariantKind::Secondary)
    }

    pub fn variant_outline(self) -> Self {
        self.variant(ButtonVariantKind::Outline)
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
            .transitions(|transitions| {
                transitions.bg(millis(200).with_easing(ease_in_out))
            })
            .children(self.children)
            .refine(self.style)
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
    spacing: Pixels,
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
            theme.size_lg,
            theme.text_size_xs,
            theme.size_3xs,
            theme.radii_md,
            px(8.),
        )
    },

    XsIcon (this, (_window, theme)) => {
        icon_size_kind(
            this,
            theme,
            theme.size_lg,
            theme.text_size_xs,
            theme.size_3xs,
            theme.radii_md,
        )
    },

    Sm (this, (window, theme)) => {
        size_kind(
            this,
            window,
            theme,
            theme.size_xl,
            theme.text_size_xs,
            theme.size_2xs,
            theme.radii_md,
            px(10.),
        )
    },

    SmIcon (this, (_window, theme)) => {
        icon_size_kind(
            this,
            theme,
            theme.size_xl,
            theme.text_size_xs,
            theme.size_2xs,
            theme.radii_md,
        )
    },

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

    MdIcon (this, (_window, theme)) => {
        icon_size_kind(
            this,
            theme,
            theme.size_2xl,
            theme.text_size_sm,
            theme.size_xs,
            theme.radii_lg,
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

    LgIcon (this, (_window, theme)) => {
        icon_size_kind(
            this,
            theme,
            theme.size_3xl,
            theme.text_size_sm,
            theme.size_xs,
            theme.radii_lg,
        )
    },
});

fn fill_button_variant_kind<E>(this: E, theme: &Theme, bg_color: Oklaba) -> E
where
    E: Styled + StatefulInteractiveElement,
{
    let fg_color = theme.fg_for_bg(ThemeFgKind::Primary, bg_color);

    this.bg(bg_color)
        .text_color(fg_color)
        .hover(|styles| styles.bg(theme.hover_color(bg_color)))
        .active(|styles| styles.bg(theme.active_color(bg_color)))
        .select_children(class("icon"), |refinement| refinement.text_color(fg_color))
}

kinds!(pub ButtonVariantKind<Styled + StatefulInteractiveElement, &Theme> {
    #[default]
    Primary (this, theme) => {
        fill_button_variant_kind(this, theme, theme.accent_primary)
    },

    Secondary (this, theme) => {
        fill_button_variant_kind(this, theme, theme.accent_secondary)
    },

    Outline (this, theme) => {
        let fg_color =
            theme.fg_for_bg(ThemeFgKind::Primary, theme.bg_secondary);

        this
            .bg(theme.bg_secondary)
            .inset_ring_1()
            .inset_ring_color(theme.bg_tertiary.lerp(&theme.bg_quaternary, 0.5))
            .text_color(fg_color)
            .hover(|styles| styles
                .bg(theme.hover_color(theme.bg_secondary))
            )
            .active(|styles| styles
                .bg(theme.active_color(theme.bg_secondary))
            )
            .select_children(class("icon"), |refinement| refinement.text_color(fg_color))
    }
});
