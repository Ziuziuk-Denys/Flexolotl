use crate::{theme::Theme, widgets::toggle};
use gpui_kit::{
    Animation, AnimationExt, AnyElement, Div, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, Stateful, Styled, div, ease_out_quint, px,
};
use std::time::Duration;

const ROW_HEIGHT: f32 = 54.;

pub fn heading(text: &'static str, theme: Theme) -> AnyElement {
    div()
        .pt(px(30.))
        .pb(px(2.))
        .text_size(px(13.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.soft)
        .child(text)
        .into_any_element()
}

pub fn note(text: &'static str, theme: Theme) -> AnyElement {
    div()
        .pt(px(8.))
        .text_size(px(13.))
        .line_height(px(18.))
        .text_color(theme.soft)
        .child(text)
        .into_any_element()
}

pub fn switch(id: &'static str, label: &'static str, is_on: bool, theme: Theme) -> Stateful<Div> {
    row(id, label, theme).child(toggle(id, is_on, theme))
}

pub fn choice(
    id: &'static str,
    label: &'static str,
    value: impl Into<SharedString>,
    theme: Theme,
) -> Stateful<Div> {
    let value = value.into();
    let key = SharedString::from(format!("{id} {value}"));
    let animation = Animation::new(Duration::from_millis(240)).with_easing(ease_out_quint());
    row(id, label, theme).child(
        div()
            .relative()
            .flex_none()
            .max_w(px(170.))
            .truncate()
            .text_color(theme.soft)
            .child(value)
            .with_animation(key, animation, |value, phase| {
                value.top(px(7. * (1. - phase))).opacity(phase)
            }),
    )
}

pub fn link(
    id: &'static str,
    label: &'static str,
    action: &'static str,
    theme: Theme,
) -> Stateful<Div> {
    choice(id, label, action, theme)
}

pub fn fact(label: &'static str, value: &'static str, theme: Theme) -> AnyElement {
    div()
        .h(px(ROW_HEIGHT))
        .flex()
        .items_center()
        .justify_between()
        .child(label)
        .child(div().text_color(theme.soft).child(value))
        .into_any_element()
}

fn row(id: &'static str, label: &'static str, theme: Theme) -> Stateful<Div> {
    div()
        .id(id)
        .min_h(px(ROW_HEIGHT))
        .flex_none()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(16.))
        .cursor_pointer()
        .border_b_1()
        .border_color(theme.hairline)
        .hover(|style| style.opacity(0.7))
        .child(div().min_w_0().child(label))
}
