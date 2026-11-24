use super::{Screen, Shell};
use crate::{assets::icon, platform::chrome, theme::Theme, widgets::icon};
use gpui_kit::{
    AnyElement, Context, Div, InteractiveElement, IntoElement, MouseButton, ParentElement,
    SharedString, Stateful, StatefulInteractiveElement, Styled, WindowControlArea, div,
    prelude::FluentBuilder, px,
};

pub const HEADER_HEIGHT: f32 = 60.;

impl Shell {
    pub(super) fn render_header(&self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        let copy = self.copy();
        let is_home = self.screen() == Screen::Home;
        let left = if is_home { copy.statuses } else { copy.back };
        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .h(px(HEADER_HEIGHT))
            .pl(px(chrome::HEADER_INSET))
            .pr(px(24.))
            .flex()
            .items_center()
            .gap(px(20.))
            .text_size(px(15.))
            .font_weight(gpui_kit::FontWeight::MEDIUM)
            .text_color(theme.ink)
            .window_control_area(WindowControlArea::Drag)
            .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move())
            .child(
                link("header-left", left).on_click(cx.listener(move |shell, _, _, cx| {
                    let target = if is_home {
                        Screen::Statuses
                    } else {
                        Screen::Home
                    };
                    shell.go_to(target, cx);
                })),
            )
            .child(div().flex_1())
            .children(is_home.then(|| {
                link("header-settings", copy.settings)
                    .on_click(cx.listener(|shell, _, _, cx| shell.go_to(Screen::Settings, cx)))
            }))
            .when(!chrome::HAS_SYSTEM_CONTROLS, |header| {
                header
                    .child(
                        control("minimize", icon::MINIMIZE, copy.minimize, theme)
                            .on_click(|_, window, _| window.minimize_window()),
                    )
                    .child(
                        control("close", icon::CLOSE, copy.close, theme)
                            .on_click(|_, window, _| window.remove_window()),
                    )
            })
            .into_any_element()
    }
}

fn link(id: &'static str, text: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .occlude()
        .cursor_pointer()
        .py(px(10.))
        .child(text)
        .hover(|style| style.opacity(0.6))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
}

fn control(
    id: &'static str,
    path: &'static str,
    label: impl Into<SharedString>,
    theme: Theme,
) -> Stateful<Div> {
    div()
        .id(id)
        .occlude()
        .aria_label(label)
        .cursor_pointer()
        .p(px(8.))
        .m(px(-8.))
        .hover(|style| style.opacity(0.6))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(icon(path, 13.).text_color(theme.ink))
}
