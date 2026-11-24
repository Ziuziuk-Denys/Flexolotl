mod activity;
mod card;
mod editor;
mod palette;
mod time;

pub use editor::CardEditor;

use crate::{presence::LinkState, shell::Shell, theme::mix, window::WIDTH};
use gpui_kit::{
    Animation, AnimationExt, AnyElement, Context, FontWeight, InteractiveElement, IntoElement,
    ParentElement, StatefulInteractiveElement, Styled, Window, div, ease_out_quint, px,
};
use std::time::Duration;

const CAPTION_TOP: f32 = 256.;
pub const CARD_TOP: f32 = 302.;
pub const CARD_WIDTH: f32 = 300.;
const CARD_LEFT: f32 = (WIDTH - CARD_WIDTH) / 2.;
pub const BUTTON_TOP: f32 = 746.;

impl Shell {
    pub fn render_home(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let card = self.render_card(window, cx);
        div()
            .relative()
            .size_full()
            .child(self.render_caption())
            .child(
                div()
                    .absolute()
                    .left(px(CARD_LEFT))
                    .top(px(self.layout.raised(CARD_TOP)))
                    .child(card),
            )
            .child(self.render_show_button(cx))
            .into_any_element()
    }

    fn render_caption(&self) -> impl IntoElement {
        let top = self.layout.raised(CAPTION_TOP);
        let theme = self.theme_at(top);
        let (key, text) = self.caption();
        div()
            .absolute()
            .left_0()
            .right_0()
            .top(px(top))
            .flex()
            .justify_center()
            .text_size(px(18.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.ink)
            .child(div().child(text).with_animation(
                ("caption", key),
                Animation::new(Duration::from_millis(320)).with_easing(ease_out_quint()),
                |caption, phase| caption.opacity(phase),
            ))
    }

    fn caption(&self) -> (usize, &'static str) {
        let copy = self.copy();
        match (&self.connection.link, self.is_live) {
            (LinkState::DiscordClosed, _) => (0, copy.open_discord),
            (LinkState::Connecting, _) => (1, copy.connecting),
            (_, true) if self.uses_music() && self.music.track.is_none() => (4, copy.nothing_plays),
            (_, true) => (2, copy.live),
            (_, false) if self.uses_music() && self.music.has_no_lyrics() => (5, copy.no_lyrics),
            (_, false) => (3, copy.preview),
        }
    }

    fn render_show_button(&self, cx: &mut Context<Self>) -> AnyElement {
        let copy = self.copy();
        let is_ready = self.is_live || self.can_show();
        let top = self.layout.anchored(BUTTON_TOP);
        let theme = self.theme_at(top);
        let text = match (self.is_live, self.is_complete()) {
            (true, _) => copy.hide,
            (false, false) => copy.fix_first,
            (false, true) => copy.show,
        };
        let outlined = if self.is_live { self.reveal() } else { 0. };
        let mut clear = theme.ink;
        clear.a = 0.;
        let fill = mix(theme.ink, clear, outlined);
        let label = mix(theme.paper, theme.ink, outlined);
        div()
            .id("show")
            .absolute()
            .left(px(CARD_LEFT))
            .top(px(top))
            .w(px(CARD_WIDTH))
            .h(px(48.))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .border_1()
            .border_color(theme.hairline)
            .bg(fill)
            .text_color(label)
            .text_size(px(16.))
            .font_weight(FontWeight::SEMIBOLD)
            .opacity(if is_ready { 1. } else { 0.4 })
            .child(text)
            .cursor_pointer()
            .hover(|style| style.opacity(if is_ready { 0.88 } else { 0.4 }))
            .on_click(cx.listener(|shell, _, _, cx| shell.toggle_live(cx)))
            .into_any_element()
    }
}
