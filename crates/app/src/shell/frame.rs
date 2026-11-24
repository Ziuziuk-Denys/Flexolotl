use super::{Screen, Shell, header::HEADER_HEIGHT, navigation::Placement};
use crate::{
    art::{Engraving, engraving},
    platform::chrome,
    theme::{self, Theme},
    tray,
    window::Layout,
};
use gpui_kit::{
    AnyElement, Context, Focusable, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    Render, Styled, Window, div, prelude::FluentBuilder, px, relative,
};
use std::time::Instant;

const HORIZON: f32 = 274.7;
const TILT: f32 = 0.2;

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.layout = Layout::of(window);
        self.advance(window);
        self.refresh_portrait(window, cx);
        let screens = self.visible_screens();
        let scene_opacity = scene_opacity(&screens);
        let backdrop = Theme::mixed(self.reveal.value.round());
        let header = self.render_header(self.theme_at(HEADER_HEIGHT / 2.), cx);
        let pages: Vec<AnyElement> = screens
            .into_iter()
            .map(|(screen, placement)| self.render_page(screen, placement, window, cx))
            .collect();
        let tray = tray::render(self, cx);
        div()
            .id("shell")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::handle_key))
            .relative()
            .size_full()
            .overflow_hidden()
            .rounded(px(chrome::CORNER))
            .bg(backdrop.paper)
            .font_family(theme::FONT)
            .text_size(px(15.))
            .when(scene_opacity > 0., |shell| {
                shell.child(self.render_scene(scene_opacity))
            })
            .children(pages)
            .child(header)
            .children(tray)
    }
}

impl Focusable for Shell {
    fn focus_handle(&self, _: &gpui_kit::App) -> gpui_kit::FocusHandle {
        self.focus.clone()
    }
}

impl Shell {
    pub fn reveal(&self) -> f32 {
        self.reveal.value
    }

    pub fn theme_at(&self, y: f32) -> Theme {
        Theme::at_height(self.reveal.value, y / self.layout.height)
    }

    fn advance(&mut self, window: &Window) {
        let seconds = self.clock.tick(Instant::now());
        self.reveal.advance(seconds);
        self.page.advance(seconds);
        self.carousel.advance(seconds);
        let is_moving =
            self.reveal.is_moving() || self.page.is_moving() || self.carousel.is_moving();
        if is_moving {
            window.request_animation_frame();
        } else {
            self.clock.rest();
        }
    }

    fn render_scene(&self, opacity: f32) -> impl IntoElement {
        div()
            .absolute()
            .size_full()
            .opacity(opacity)
            .child(engraving(Engraving {
                portrait: self.portrait.image.clone(),
                lines: self.settings.lines,
                reveal: self.reveal.value,
                tilt: TILT,
                horizon: self.layout.raised(HORIZON),
            }))
    }

    fn render_page(
        &mut self,
        screen: Screen,
        placement: Placement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let content = match screen {
            Screen::Home => self.render_home(window, cx),
            Screen::Statuses => self.render_statuses(cx),
            Screen::Settings => self.render_settings(cx),
            Screen::Plugins => self.render_plugins(cx),
        };
        div()
            .absolute()
            .top_0()
            .left(px(placement.offset))
            .w(relative(1.))
            .h_full()
            .opacity(placement.opacity)
            .child(content)
            .into_any_element()
    }

    fn handle_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "escape" if self.tray.is_some() => self.close_tray(window, cx),
            "escape" => self.go_home(cx),
            "left" if self.screen() == Screen::Statuses => self.carousel.step(-1, cx),
            "right" if self.screen() == Screen::Statuses => self.carousel.step(1, cx),
            "enter" if self.screen() == Screen::Statuses => self.open_centred(window, cx),
            _ => {}
        }
    }
}

fn scene_opacity(screens: &[(Screen, Placement)]) -> f32 {
    screens
        .iter()
        .find(|(screen, _)| *screen == Screen::Home)
        .map_or(0., |(_, placement)| placement.opacity)
}
