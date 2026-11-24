use super::icon;
use crate::assets::icon as path;
use gpui_kit::{Hsla, IntoElement, ParentElement, Styled, div, px};

pub fn face(size: f32, is_awake: bool, head: Hsla, gills: Hsla) -> impl IntoElement {
    let head_path = if is_awake {
        path::FACE_AWAKE
    } else {
        path::FACE_ASLEEP
    };
    div()
        .relative()
        .size(px(size))
        .flex_none()
        .child(icon(path::GILLS, size).absolute().text_color(gills))
        .child(icon(head_path, size).absolute().text_color(head))
}
