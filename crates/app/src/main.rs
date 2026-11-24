#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod arguments;
mod art;
mod assets;
mod background;
mod copy;
mod home;
mod motion;
mod music;
mod pictures;
mod platform;
mod plugins;
mod preferences;
mod presence;
mod saver;
mod shell;
mod statuses;
mod theme;
mod tray;
mod update;
mod widgets;
mod window;

use arguments::Arguments;
use gpui_kit::QuitMode;
use platform::instance::{self, Claim};
use reqwest_client::ReqwestClient;
use std::sync::Arc;

fn main() {
    let arguments = Arguments::read();
    let (sender, requests) = async_channel::unbounded();
    let claim = instance::claim(arguments, sender.clone());
    let Claim::Accepted(predecessor) = claim else {
        return;
    };
    let http_client = ReqwestClient::user_agent("Flexolotl").expect("valid user agent");
    let application = gpui_kit::application()
        .with_quit_mode(QuitMode::Explicit)
        .with_assets(assets::Assets)
        .with_http_client(Arc::new(http_client));
    application.on_reopen(background::open);
    application.run(move |cx| {
        gpui_kit::init(cx);
        assets::register_fonts(cx);
        #[cfg(target_os = "macos")]
        platform::menu::install(cx);
        background::start(sender, requests, arguments, predecessor, cx);
    });
}
