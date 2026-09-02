use gpui::{
    Element, Refineable, StatefulInteractiveElement, StyleRefinement, Styled,
};

use crate::Kind;

pub trait StyledElement:
    Element + Styled + StatefulInteractiveElement + Sized
{
    fn apply_kind<'a, K: Kind>(self, kind: K, data: K::Data<'a>) -> Self {
        kind.apply(self, data)
    }

    fn refine(mut self, refinement: StyleRefinement) -> Self {
        self.style().refine(&refinement);
        self
    }
}

impl<E: Element + Styled + StatefulInteractiveElement> StyledElement for E {}
