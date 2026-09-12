use gpui::{StyleRefinement, Styled};
use tesserae_macros::Styles;
use tesserae_utils::{StyledElement, StylesEnum};

#[derive(Default)]
struct TestElement {
    style: StyleRefinement,
}

impl Styled for TestElement {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

#[derive(Clone, Copy, Styles)]
#[styles_data(&f32)]
enum TestStyles {
    #[styles(|refinement, opacity| refinement.opacity(*opacity))]
    PrimaryAction,

    #[class("quiet")]
    #[styles(|refinement, opacity| refinement.opacity(*opacity / 2.0))]
    SecondaryAction,
}

#[test]
fn derives_classes_and_refinements() {
    assert_eq!(TestStyles::PrimaryAction.class_name(), "primary_action");
    assert_eq!(TestStyles::SecondaryAction.class_name(), "quiet");

    let element = TestElement::default()
        .refine_styles_with_enum(TestStyles::SecondaryAction, &0.8);
    let expected_class = StyleRefinement::default().class("quiet");

    assert_eq!(element.style.opacity, Some(0.4));
    assert_eq!(
        element.style.selector_state(),
        expected_class.selector_state()
    );
}

#[derive(Clone, Copy, Styles)]
enum UnitStyles {
    #[styles(|refinement, ()| refinement.flex())]
    Compact,
}

#[test]
fn defaults_styles_data_to_unit() {
    let element =
        TestElement::default().refine_styles_with_enum(UnitStyles::Compact, ());

    assert_eq!(element.style.display, Some(gpui::Display::Flex));
}
