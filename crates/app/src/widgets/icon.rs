use gpui_kit::{Styled, Svg, px, svg};

pub fn icon(path: &'static str, size: f32) -> Svg {
    svg().path(path).size(px(size)).flex_none()
}
