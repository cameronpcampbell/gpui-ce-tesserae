use gpui::{
    AbsoluteLength, App, DefiniteLength, ElementId, FocusHandle, Pixels, Window,
};

pub fn focus_handle(
    base_id: impl Into<ElementId>,
    window: &mut Window,
    cx: &mut App,
) -> FocusHandle {
    window
        .use_keyed_state(
            (base_id.into(), "state:focus_handle"),
            cx,
            |_window, cx| cx.focus_handle().tab_stop(true),
        )
        .read(cx)
        .clone()
}

pub trait WindowUtils {
    /// Returns the vertical padding to apply on each side of a component so its
    /// text line box reaches `target_height`.
    ///
    /// `text_size` resolves relative `line_height` values before the line height
    /// is snapped to the physical pixel grid.
    fn padding_for_height(
        &self,
        target_height: impl Into<AbsoluteLength>,
        text_size: impl Into<AbsoluteLength>,
        line_height: impl Into<DefiniteLength>,
    ) -> Pixels;
}

impl WindowUtils for Window {
    fn padding_for_height(
        &self,
        target_height: impl Into<AbsoluteLength>,
        text_size: impl Into<AbsoluteLength>,
        line_height: impl Into<DefiniteLength>,
    ) -> Pixels {
        let rem_size = self.rem_size();

        let target_height = target_height.into().to_pixels(rem_size);
        let text_size = text_size.into().to_pixels(rem_size);
        let line_height = self
            .pixel_snap(line_height.into().to_pixels(text_size.into(), rem_size));

        (target_height - line_height) / 2.
    }
}
