mod carousel;
mod cover;

pub use carousel::Carousel;

use crate::{shell::Shell, theme::Theme, window::WIDTH};
use cover::COVER;
use flexolotl_core::status::Status;
use gpui_kit::{
    AnyElement, Context, CursorStyle, FontWeight, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, ScrollEvent, Styled, Window, div,
    px,
};

const LEFT: f32 = (WIDTH - COVER) / 2.;
const TITLE_TOP: f32 = 92.;
const TRACK_TOP: f32 = 176.;

impl Shell {
    pub fn render_statuses(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::mixed(self.reveal().round());
        let copy = self.copy();
        let count = self.deck.statuses().len();
        let mut covers: Vec<AnyElement> = (0..count)
            .map(|index| self.render_cover(index, theme, cx))
            .collect();
        covers.push(self.render_new_cover(count, theme, cx));
        let cursor = if self.carousel.is_dragged() {
            CursorStyle::ClosedHand
        } else {
            CursorStyle::OpenHand
        };
        div()
            .id("statuses")
            .relative()
            .size_full()
            .cursor(cursor)
            .text_color(theme.ink)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::grab_statuses))
            .on_mouse_move(cx.listener(Self::drag_statuses))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::release_statuses))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::release_statuses))
            .on_scroll(cx.listener(Self::scroll_statuses))
            .child(
                div()
                    .absolute()
                    .top(px(TITLE_TOP))
                    .left(px(LEFT))
                    .text_size(px(28.))
                    .line_height(px(36.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(copy.statuses),
            )
            .children(covers)
            .into_any_element()
    }

    pub fn open_centred(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_cover(self.carousel.centred(), window, cx);
    }

    fn open_cover(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index < self.deck.statuses().len() {
            self.open_status(index, window, cx);
        } else {
            self.add_status(Status::default(), window, cx);
        }
    }

    fn grab_statuses(&mut self, event: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.carousel.grab(event.position.x.into());
        cx.notify();
    }

    fn drag_statuses(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        let last = self.deck.statuses().len();
        if event.pressed_button == Some(MouseButton::Left)
            && self.carousel.drag(event.position.x.into(), last)
        {
            cx.notify();
        }
    }

    fn release_statuses(
        &mut self,
        event: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let last = self.deck.statuses().len();
        let Some(was_click) = self.carousel.release(last) else {
            return;
        };
        let y: f32 = event.position.y.into();
        let is_on_cover = (TRACK_TOP..TRACK_TOP + COVER).contains(&y);
        if was_click && is_on_cover {
            let x: f32 = event.position.x.into();
            if let Some(index) = self.carousel.index_at(x - LEFT, COVER, last) {
                if index == self.carousel.centred() {
                    self.open_cover(index, window, cx);
                } else {
                    self.carousel.go_to(index);
                }
            }
        }
        cx.notify();
    }

    fn scroll_statuses(
        &mut self,
        event: &ScrollEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let delta = event.delta.pixel_delta(window.line_height());
        let amount: f32 = if delta.x.abs() > delta.y.abs() {
            delta.x.into()
        } else {
            delta.y.into()
        };
        let last = self.deck.statuses().len();
        if self.carousel.scroll(amount, last) {
            cx.notify();
        }
    }
}
