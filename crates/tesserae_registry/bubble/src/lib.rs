use gpui::{
    AnyElement, ElementId, FontWeight, StyleRefinement, div, prelude::*, px,
};
use palette::Oklaba;
use smallvec::SmallVec;
use tesserae_macros::Styles;
use tesserae_theme::Theme;
use tesserae_utils::{StyledElement, WindowUtils};

#[derive(IntoElement)]
pub struct Bubble {
    element_id: ElementId,
    variant: BubbleVariantKind,
    anchor: BubbleAnchorKind,
    children: SmallVec<[AnyElement; 2]>,
    style: StyleRefinement,
}

impl ParentElement for Bubble {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Bubble {
    pub fn new(element_id: impl Into<ElementId>) -> Self {
        Self {
            element_id: element_id.into(),
            variant: BubbleVariantKind::default(),
            anchor: BubbleAnchorKind::default(),
            children: SmallVec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for Bubble {
    fn render(
        self,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> impl gpui::IntoElement {
        let theme = Theme::read_global(cx);

        div()
            .id(self.element_id)
            .rounded_full()
            .rounded_smoothing_1()
            .border_1()
            .items_center()
            .justify_center()
            .flex()
            .line_height(theme.line_height)
            .px(px(10.))
            .py(window.padding_for_height(
                theme.size_9xl,
                theme.text_size_sm,
                theme.line_height,
            ) - px(2.))
            .text_size(theme.text_size_sm)
            .text_color(theme.fg_primary)
            .font_family("Geist")
            .font_weight(FontWeight::NORMAL)
            .children(self.children)
            .refine_style_with_enum(self.variant, theme)
            .refine_style_with_enum(self.anchor, theme)
            .refine_style(&self.style)
    }
}

impl Styled for Bubble {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

fn variant_styles(
    refinement: StyleRefinement,
    background: Oklaba,
    ring_color: Oklaba,
) -> StyleRefinement {
    refinement.bg(background).border_color(ring_color)
}

#[derive(Clone, Copy, Default, Styles)]
#[styles_data(&Theme)]
pub enum BubbleVariantKind {
    #[styles(|refinement, theme| {
        variant_styles(
            refinement,
            theme.bg_primary,
            theme.bg_secondary
        )
    })]
    Primary,

    #[default]
    #[styles(|refinement, theme| {
        variant_styles(
            refinement,
            theme.bg_secondary,
            theme.bg_tertiary
        )
    })]
    Secondary,

    #[styles(|refinement, theme| {
        variant_styles(
            refinement,
            theme.bg_tertiary,
            theme.bg_quaternary,
        )
    })]
    Tertiary,

    #[styles(|refinement, theme| {
        variant_styles(
            refinement,
            theme.bg_quaternary,
            theme.bg_quinary,
        )
    })]
    Quaternary,

    #[styles(|refinement, theme| {
        variant_styles(
            refinement,
            theme.bg_quinary,
            theme.bg_senary,
        )
    })]
    Quinary,
}

#[derive(Clone, Copy, Default, Styles)]
#[styles_data(&Theme)]
pub enum BubbleAnchorKind {
    #[styles(|refinement, theme| {
        refinement.rounded_tl(theme.radii_md)
    })]
    TopLeft,

    #[styles(|refinement, theme| {
        refinement
            .rounded_tl(theme.radii_md)
            .rounded_bl(theme.radii_md)
    })]
    CenterLeft,

    #[styles(|refinement, theme| {
        refinement.rounded_bl(theme.radii_md)
    })]
    BottomLeft,

    #[styles(|refinement, theme| {
        refinement.rounded_tr(theme.radii_md)
    })]
    TopRight,

    #[styles(|refinement, theme| {
        refinement
            .rounded_tr(theme.radii_md)
            .rounded_br(theme.radii_md)
    })]
    CenterRight,

    #[default]
    #[styles(|refinement, theme| {
        refinement.rounded_br(theme.radii_md)
    })]
    BottomRight,
}
