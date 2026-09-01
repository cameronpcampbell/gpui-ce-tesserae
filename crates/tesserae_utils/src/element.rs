use gpui::{Element, StatefulInteractiveElement, Styled};

use crate::Variant;

pub trait StyledElement:
    Element + Styled + StatefulInteractiveElement + Sized
{
    fn apply_variant<'a, V: Variant>(self, variant: V, data: V::Data<'a>) -> Self {
        variant.apply(self, data)
    }
}

impl<E: Element + Styled + StatefulInteractiveElement> StyledElement for E {}
