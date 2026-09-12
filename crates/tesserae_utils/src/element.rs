use gpui::{Refineable, StyleRefinement, Styled};

use crate::StylesEnum;

pub trait StyledElement: Styled + Sized {
    fn refine_style_with_enum<'a, Style: StylesEnum>(
        self,
        styles: Style,
        data: Style::Data<'a>,
    ) -> Self {
        let class_name = StylesEnum::class_name(&styles);
        let refinement =
            StylesEnum::refine(&styles, StyleRefinement::default(), data);

        self.class(class_name).refine_style(&refinement)
    }

    fn refine_style(mut self, refinement: &StyleRefinement) -> Self {
        self.style().refine(refinement);

        self
    }
}

impl<Element: Styled> StyledElement for Element {}
