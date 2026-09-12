use std::rc::Rc;

use gpui::{
    App, DurationWithEasing, ElementId, InteractiveElement, IntoElement, Lerp,
    ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, div, ease_in_out, linear_color_stop,
    linear_gradient, millis, prelude::FluentBuilder, selectors::class,
};
use palette::WithAlpha;
use tesserae_base::switch::{BaseSwitch, BaseSwitchThumb, OnChange};
use tesserae_theme::Theme;
use tesserae_utils::{PerceptualColor, StyledElement};

#[derive(IntoElement)]
pub struct Slider {
    element_id: ElementId,
    style: StyleRefinement,
}

impl Slider {
    pub fn new(element_id: impl Into<ElementId>) -> Self {
        Self {
            element_id: element_id.into(),
            style: StyleRefinement::default(),
        }
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::read_global(cx);

        div()
            .h(theme.size_xs)
            .rounded_full()
            .rounded_smoothing_1()
            .bg(theme.bg_secondary)
            .w_full()
            .refine_style(&self.style)
    }
}

impl Styled for Slider {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
