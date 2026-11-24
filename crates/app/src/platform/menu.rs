use crate::background;
use gpui_kit::{App, KeyBinding, Menu, MenuItem, actions};

actions!(flexolotl, [CloseWindow, Quit]);

pub fn install(cx: &mut App) {
    cx.on_action(|_: &Quit, cx| background::quit(cx));
    cx.on_action(|_: &CloseWindow, cx| background::close(cx));
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-w", CloseWindow, None),
    ]);
    cx.set_menus([Menu::new("Flexolotl").items([
        MenuItem::action("Close Window", CloseWindow),
        MenuItem::separator(),
        MenuItem::action("Quit Flexolotl", Quit),
    ])]);
}
