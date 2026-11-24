use crate::{platform::chrome, shell::Shell};
use gpui_kit::{
    AnyWindowHandle, App, Bounds, Entity, TitlebarOptions, WindowBounds, WindowDecorations,
    WindowOptions, point, px, size,
};

pub const WIDTH: f32 = 440.;
pub const HEIGHT: f32 = 820.;

pub fn open(shell: Entity<Shell>, cx: &mut App) -> AnyWindowHandle {
    let bounds = Bounds::centered(None, size(px(WIDTH), px(HEIGHT)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(WIDTH), px(HEIGHT))),
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
