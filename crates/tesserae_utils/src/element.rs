use gpui::{Refineable, StyleRefinement, Styled};

use crate::Kind;

pub trait StyledElement: Styled + Sized {
    fn apply_kind<'a, K: Kind<Self>>(self, kind: K, data: K::Data<'a>) -> Self {
        kind.apply(self, data)
    }

    fn refine(mut self, refinement: StyleRefinement) -> Self {
        self.style().refine(&refinement);
        self
    }
}

impl<E: Styled> StyledElement for E {}
