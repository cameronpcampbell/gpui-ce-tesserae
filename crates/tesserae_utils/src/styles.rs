use gpui::StyleRefinement;

/// An enum whose variants produce style refinements and element classes.
pub trait StylesEnum {
    type Data<'a>;

    /// Returns the class associated with the current variant.
    fn class_name(&self) -> &'static str;

    /// Builds the style refinement associated with the current variant.
    fn refine<'a>(
        &self,
        refinement: StyleRefinement,
        data: Self::Data<'a>,
    ) -> StyleRefinement;
}
