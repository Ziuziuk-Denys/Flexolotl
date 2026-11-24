use flexolotl_core::settings::Lines;
use gpui_kit::{Corners, IntoElement, RenderImage, Styled, canvas, px};
use std::sync::Arc;

pub struct Engraving {
    pub portrait: Arc<RenderImage>,
    pub lines: Lines,
    pub reveal: f32,
    pub tilt: f32,
    pub horizon: f32,
}

pub fn engraving(engraving: Engraving) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let scale = window.scale_factor();
            let parameters = Corners {
                top_left: px(-(1. + engraving.reveal) / scale),
                top_right: px(-spacing(engraving.lines)),
                bottom_right: px(-(2. + engraving.tilt) / scale),
                bottom_left: px(-engraving.horizon - 1. / scale),
            };
            let portrait = engraving.portrait.clone();
            let _ = window.paint_image(bounds, bounds, parameters, portrait, 0, false);
        },
    )
    .absolute()
    .size_full()
}

fn spacing(lines: Lines) -> f32 {
    match lines {
        Lines::Fine => 4.,
        Lines::Medium => 5.5,
        Lines::Bold => 8.,
    }
}
