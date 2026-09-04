use gpui::{
    AnyElement, ElementId, FontWeight, StyleRefinement, div, prelude::*, px,
};
use palette::Oklaba;
use smallvec::SmallVec;
use tesserae_theme::Theme;
use tesserae_utils::{StyledElement, WindowUtils, kinds};

#[derive(IntoElement)]
pub struct Bubble {
    id: ElementId,
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
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
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
            .id(self.id)
            .rounded_full()
            .rounded_smoothing_1()
            .border_1()
            .items_center()
            .justify_center()
            .flex()
            .line_height(theme.line_height)
            .px(px(10.))
            .py(window.padding_for_height(
                theme.size_3xl,
                theme.text_size_sm,
                theme.line_height,
            ) - px(2.))
            .text_size(theme.text_size_sm)
            .text_color(theme.fg_primary)
            .font_family("Geist")
            .font_weight(FontWeight::NORMAL)
            .children(self.children)
            .apply_kind(self.variant, theme)
            .apply_kind(self.anchor, theme)
            .refine(self.style)
    }
}

impl Styled for Bubble {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

fn variant_kind<E: Styled>(this: E, bg: Oklaba, ring_color: Oklaba) -> E {
    this.bg(bg).border_color(ring_color)
}

kinds!(pub BubbleVariantKind<_, &Theme> {
    Primary (this, theme) => {
        variant_kind(
            this,
            theme.bg_primary,
            theme.bg_secondary
        )
    },

    #[default]
    Secondary (this, theme) => {
        variant_kind(
            this,
            theme.bg_secondary,
            theme.bg_tertiary
        )
    },

    Tertiary (this, theme) => {
        variant_kind(
            this,
            theme.bg_tertiary,
            theme.bg_quaternary,
        )
    },

    Quaternary (this, theme) => {
        variant_kind(
            this,
            theme.bg_quaternary,
            theme.bg_quinary,
        )
    },

    Quinary (this, theme) => {
        variant_kind(
            this,
            theme.bg_quinary,
            theme.bg_senary,
        )
    },
});

kinds!(pub BubbleAnchorKind<_, &Theme> {
    TopLeft (this, theme) => {
        this.rounded_tl(theme.radii_md)
    },

    CenterLeft (this, theme) => {
        this.rounded_tl(theme.radii_md).rounded_bl(theme.radii_md)
    },

    BottomLeft (this, theme) => {
        this.rounded_bl(theme.radii_md)
    },

    TopRight (this, theme) => {
        this.rounded_tr(theme.radii_md)
    },

    CenterRight (this, theme) => {
        this.rounded_tr(theme.radii_md).rounded_br(theme.radii_md)
    },

    #[default]
    BottomRight (this, theme) => {
        this.rounded_br(theme.radii_md)
    },
});
