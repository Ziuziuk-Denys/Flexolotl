#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod art;
mod assets;
mod background;
mod copy;
mod home;
mod motion;
mod music;
mod pictures;
mod platform;
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

use gpui_kit::QuitMode;
use platform::instance::{self, Instance};
use reqwest_client::ReqwestClient;
use std::{env, sync::Arc};

pub const BACKGROUND_FLAG: &str = "--background";
pub const AFTER_UPDATE_FLAG: &str = "--after-update";
pub const LIVE_FLAG: &str = "--live";

fn main() {
    if has_flag(AFTER_UPDATE_FLAG) {
        instance::wait_for_previous();
    }
    let (sender, requests) = async_channel::unbounded();
    let Instance::First = instance::claim(sender.clone()) else {
        return;
    };
    let starts_hidden = has_flag(BACKGROUND_FLAG);
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
        background::start(sender, requests, starts_hidden, cx);
    });
}

pub fn has_flag(flag: &str) -> bool {
    env::args().skip(1).any(|argument| argument == flag)
}
