use gpui::{
    DurationWithEasing, ElementId, FocusHandle, InteractiveElement, IntoElement,
    Pixels, RenderOnce, StyleRefinement, Styled, div, ease_in_out, millis,
    prelude::FluentBuilder, px,
};
use palette::WithAlpha;
use tesserae_utils::StyledElement;

use tesserae_theme::Theme;

#[derive(IntoElement)]
pub struct FocusRing {
    id: ElementId,
    thickness: Pixels,
    style: StyleRefinement,
    focus_handle: FocusHandle,
}

impl FocusRing {
    pub fn new(id: impl Into<ElementId>, focus_handle: FocusHandle) -> Self {
        Self {
            id: id.into(),
            thickness: px(3.),
            style: StyleRefinement::default(),
            focus_handle,
        }
    }
}

impl RenderOnce for FocusRing {
    fn render(
        self,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> impl gpui::IntoElement {
        let theme = Theme::read_global(cx);

        div()
            .id(self.id)
            .absolute()
            .inset_0()
            .rounded_smoothing_1()
            .ring(self.thickness)
            .ring_color(theme.accent_primary.with_alpha(0.))
            .inset(px(-6.))
            .refine(self.style)
            .transitions(|transitions| {
                transitions
                    .inset(millis(120).with_easing(ease_in_out))
                    .ring_color(millis(120).with_easing(ease_in_out))
                    .rounded(millis(120).with_easing(ease_in_out))
            })
            .when(self.focus_handle.is_focused(window), |ring| {
                ring.inset_0()
                    .ring_color(theme.accent_primary.with_alpha(0.3))
            })
    }
}

impl Styled for FocusRing {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
