use crate::{platform::chrome, shell::Shell};
use gpui_kit::{
    AnyWindowHandle, App, Bounds, Entity, TitlebarOptions, Window, WindowBounds, WindowDecorations,
    WindowOptions, point, px, size,
};

pub const WIDTH: f32 = 440.;
const HEIGHT: f32 = 820.;
const MIN_HEIGHT: f32 = 624.;
const SCREEN_MARGIN: f32 = 72.;
const MOST_SQUEEZED: f32 = 170.;

pub fn open(shell: Entity<Shell>, cx: &mut App) -> AnyWindowHandle {
    let room = screen_height(cx).map_or(HEIGHT, |height| height - SCREEN_MARGIN);
    let size = size(px(WIDTH), px(fitting_height(room)));
    let bounds = Bounds::centered(None, size, cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size),
        window_background: chrome::BACKGROUND,
        window_decorations: Some(WindowDecorations::Client),
        is_resizable: false,
        app_id: Some("quest.nuvo.Flexolotl".into()),
        titlebar: Some(TitlebarOptions {
            title: Some("Flexolotl".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(20.), px(23.))),
        }),
        ..WindowOptions::default()
    };
    let (window, _) = gpui_kit::open_window(options, cx, |window, cx| {
        shell.update(cx, |shell, cx| shell.attach(window, cx));
        shell
    })
    .expect("a window can be opened");
    window
}

fn screen_height(cx: &App) -> Option<f32> {
    let displays = cx
        .primary_display()
        .map_or_else(|| cx.displays(), |display| vec![display]);
    displays
        .iter()
        .map(|display| f32::from(display.visible_bounds().size.height))
        .reduce(f32::min)
}

fn fitting_height(room: f32) -> f32 {
    room.clamp(MIN_HEIGHT, HEIGHT)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub height: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self { height: HEIGHT }
    }
}

impl Layout {
    pub fn of(window: &Window) -> Self {
        Self {
            height: window.viewport_size().height.into(),
        }
    }

    pub fn raised(self, y: f32) -> f32 {
        y - self.missing().min(MOST_SQUEEZED)
    }

    pub fn anchored(self, y: f32) -> f32 {
        y - self.missing()
    }

    fn missing(self) -> f32 {
        (HEIGHT - self.height).max(0.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fits_a_low_screen() {
        assert_eq!(fitting_height(1032.), HEIGHT);
        assert_eq!(fitting_height(680.), 680.);
        assert_eq!(fitting_height(400.), MIN_HEIGHT);
    }

    #[test]
    fn keeps_the_button_below_the_card_in_the_lowest_window() {
        let layout = Layout { height: MIN_HEIGHT };
        let tallest_card = 412.;
        let card_bottom = layout.raised(crate::home::CARD_TOP) + tallest_card;
        assert!(card_bottom < layout.anchored(crate::home::BUTTON_TOP));
    }
}
