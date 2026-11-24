mod application;
mod button;
mod options;
mod picture;

pub use button::ButtonTray;

use crate::{
    shell::Shell,
    theme::{self, Theme},
    widgets::{FieldColors, TextField},
};
use flexolotl_core::activity::ActivityKind;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, Context, Div, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, SharedString, Stateful, Styled, div, ease_out_quint, px, rgba,
};
use std::time::Duration;

const RADIUS: f32 = 20.;

pub enum Tray {
    Verb,
    Time,
    Button(ButtonTray),
    Picture(TextField),
    Application(TextField),
}

impl Shell {
    pub fn open_tray(&mut self, tray: Tray, cx: &mut Context<Self>) {
        self.tray = Some(tray);
        cx.notify();
    }
}

pub fn render(shell: &Shell, cx: &mut Context<Shell>) -> Option<AnyElement> {
    let copy = shell.copy();
    let (title, content) = match shell.tray.as_ref()? {
        Tray::Verb => (copy.verb_title, options::verbs(shell, cx)),
        Tray::Time => (copy.time_title, options::times(shell, cx)),
        Tray::Button(tray) => (copy.button_title, tray.render(copy, cx)),
        Tray::Picture(field) => (copy.picture_title, picture::render(shell, field, cx)),
        Tray::Application(field) => (copy.application_title, application::render(copy, field, cx)),
    };
    let sheet = div()
        .id("tray")
        .absolute()
        .left_0()
        .right_0()
        .occlude()
        .p(px(28.))
        .rounded(px(RADIUS))
        .bg(theme::sheet())
        .text_color(Theme::day().ink)
        .flex()
        .flex_col()
        .child(
            div()
                .text_size(px(24.))
                .line_height(px(32.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
        .child(content)
        .with_animation(
            "tray-rise",
            Animation::new(Duration::from_millis(280)).with_easing(ease_out_quint()),
            |sheet, phase| sheet.bottom(px(-36. * (1. - phase))).opacity(phase),
        );
    let veil = div()
        .id("veil")
        .absolute()
        .inset_0()
        .rounded(px(RADIUS))
        .occlude()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|shell, _, window, cx| shell.close_tray(window, cx)),
        )
        .child(sheet)
        .with_animation(
            "veil-fade",
            Animation::new(Duration::from_millis(200)),
            |veil, phase| veil.bg(rgba((0x58 as f32 * phase) as u32)),
        );
    Some(veil.into_any_element())
}

fn field_colors() -> FieldColors {
    let day = Theme::day();
    FieldColors {
        ink: day.ink.into(),
        soft: rgba(0x68665fb0).into(),
        selection: rgba(0x13131226).into(),
    }
}

fn labelled(label: &'static str, field: &TextField) -> Div {
    let day = Theme::day();
    div()
        .mt(px(18.))
        .flex()
        .flex_col()
        .child(div().text_size(px(13.)).text_color(day.soft).child(label))
        .child(
            div()
                .h(px(42.))
                .flex()
                .items_center()
                .border_b_1()
                .border_color(rgba(0x14141233))
                .text_size(px(18.))
                .child(field.element()),
        )
}

fn problem(text: &'static str) -> Div {
    div()
        .mt(px(6.))
        .text_size(px(13.))
        .text_color(theme::danger())
        .child(text)
}

fn done_button(text: impl Into<SharedString>) -> Stateful<Div> {
    let day = Theme::day();
    div()
        .id("done")
        .mt(px(24.))
        .h(px(48.))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .bg(day.ink)
        .text_color(theme::sheet())
        .text_size(px(16.))
        .font_weight(FontWeight::SEMIBOLD)
        .hover(|style| style.opacity(0.88))
        .child(text.into())
}

fn outlined_button(id: &'static str, text: impl Into<SharedString>) -> Stateful<Div> {
    div()
        .id(id)
        .h(px(48.))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .border_1()
        .border_color(rgba(0x14141240))
        .text_size(px(16.))
        .font_weight(FontWeight::MEDIUM)
        .hover(|style| style.bg(rgba(0x1414120a)))
        .child(text.into())
}

fn remove_link(text: &'static str) -> Stateful<Div> {
    quiet_link("remove", text).text_color(theme::danger())
}

fn quiet_link(id: &'static str, text: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .mt(px(16.))
        .flex()
        .justify_center()
        .cursor_pointer()
        .hover(|style| style.opacity(0.7))
        .child(text)
}

pub fn kind_icon(kind: ActivityKind) -> &'static str {
    use crate::assets::icon;
    match kind {
        ActivityKind::Playing => icon::PLAYING,
        ActivityKind::Listening => icon::LISTENING,
        ActivityKind::Watching => icon::WATCHING,
        ActivityKind::Competing => icon::COMPETING,
    }
}
