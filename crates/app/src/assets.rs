use gpui_kit::{App, AssetSource, Result, SharedString};
use std::borrow::Cow;

pub struct Assets;

macro_rules! icons {
    ($($constant:ident = $file:literal),* $(,)?) => {
        pub mod icon {
            $(pub const $constant: &str = concat!("icons/", $file);)*
        }

        fn bytes(path: &str) -> Option<&'static [u8]> {
            match path {
                $(icon::$constant => Some(include_bytes!(concat!("../assets/icons/", $file))),)*
                _ => None,
            }
        }
    };
}

icons! {
    CLOSE = "close.svg",
    MINIMIZE = "minimize.svg",
    PLUS = "plus.svg",
    PLAYING = "game.svg",
    LISTENING = "listen.svg",
    WATCHING = "watch.svg",
    COMPETING = "compete.svg",
    MUSIC = "music.svg",
    PLUGIN = "plugin.svg",
    FACE_ASLEEP = "mark-head-32.svg",
    FACE_AWAKE = "mark-awake-head-32.svg",
    GILLS = "mark-gills-32.svg",
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(bytes(path).map(Cow::Borrowed))
    }

    fn list(&self, _: &str) -> Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

pub fn register_fonts(cx: &App) {
    let fonts: [&'static [u8]; 5] = [
        include_bytes!("../assets/fonts/GolosText-400.ttf"),
        include_bytes!("../assets/fonts/GolosText-500.ttf"),
        include_bytes!("../assets/fonts/GolosText-600.ttf"),
        include_bytes!("../assets/fonts/NotoSans-400.ttf"),
        include_bytes!("../assets/fonts/NotoSans-600.ttf"),
    ];
    cx.text_system()
        .add_fonts(fonts.map(Cow::Borrowed).to_vec())
        .expect("bundled fonts are valid");
}
