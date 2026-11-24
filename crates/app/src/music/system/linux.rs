use super::is_browser;
use crate::music::Track;
use std::{collections::HashMap, fs};
use zbus::{
    blocking::{Connection, Proxy},
    zvariant::OwnedValue,
};

const PREFIX: &str = "org.mpris.MediaPlayer2.";
const ROOT: &str = "org.mpris.MediaPlayer2";
const PLAYER: &str = "org.mpris.MediaPlayer2.Player";
const PATH: &str = "/org/mpris/MediaPlayer2";
const PROXY: &str = "playerctld";
const BUS: &str = "org.freedesktop.DBus";
const ENGINE_NAMES: [&str; 3] = ["chromium", "chrome", "electron"];

type Values = HashMap<String, OwnedValue>;

pub struct Media {
    bus: Connection,
}

impl Media {
    pub fn connect() -> Option<Self> {
        Connection::session().ok().map(|bus| Self { bus })
    }

    pub fn playing(&self, hears_browsers: bool) -> Option<Track> {
        let names: Vec<String> = self.bus_proxy()?.call("ListNames", &()).ok()?;
        let mut tracks: Vec<Track> = names
            .iter()
            .filter(|name| name.starts_with(PREFIX) && !name.ends_with(PROXY))
            .filter_map(|name| self.read_track(name))
            .filter(|track| hears_browsers || !track.is_from_browser)
            .collect();
        tracks.sort_by_key(|track| track.is_from_browser);
        tracks.into_iter().next()
    }

    fn read_track(&self, name: &str) -> Option<Track> {
        let properties =
            Proxy::new(&self.bus, name, PATH, "org.freedesktop.DBus.Properties").ok()?;
        let state: Values = properties.call("GetAll", &(PLAYER,)).ok()?;
        if text(&state, "PlaybackStatus")? != "Playing" {
            return None;
        }
        let metadata: Values = state.get("Metadata")?.try_clone().ok()?.try_into().ok()?;
        let title = text(&metadata, "xesam:title").filter(|title| !title.trim().is_empty())?;
        let position_us = number(&state, "Position").unwrap_or(0);
        let length_us = number(&metadata, "mpris:length").unwrap_or(0);
        let program = self.program(name);
        let page = text(&metadata, "xesam:url");
        let track = Track::new(
            title,
            artists(&metadata),
            self.player_name(name, &program),
            length_us / 1000,
        )
        .heard_in_browser(is_browser(&program))
        .with_cover(text(&metadata, "mpris:artUrl"))
        .on_page(page.as_deref())
        .at(position_us / 1000);
        Some(track)
    }

    fn player_name(&self, name: &str, program: &str) -> String {
        let identity = Proxy::new(&self.bus, name, PATH, ROOT)
            .ok()
            .and_then(|proxy| proxy.get_property::<String>("Identity").ok())
            .unwrap_or_default();
        let is_generic = ENGINE_NAMES.contains(&identity.to_lowercase().as_str());
        if identity.is_empty() || is_generic && !is_browser(program) || identity.starts_with('@') {
            capitalized(program)
        } else {
            identity
        }
    }

    fn program(&self, name: &str) -> String {
        let process: Option<u32> = self
            .bus_proxy()
            .and_then(|bus| bus.call("GetConnectionUnixProcessID", &(name,)).ok());
        process
            .and_then(|process| {
                let program = fs::read_link(format!("/proc/{process}/exe")).ok()?;
                let file = program.file_name()?.to_string_lossy().into_owned();
                Some(file)
            })
            .unwrap_or_else(|| {
                let short = &name[PREFIX.len()..];
                short.split('.').next().unwrap_or(short).to_owned()
            })
    }

    fn bus_proxy(&self) -> Option<Proxy<'_>> {
        Proxy::new(&self.bus, BUS, "/org/freedesktop/DBus", BUS).ok()
    }
}

fn capitalized(program: &str) -> String {
    let mut letters = program.chars();
    letters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(letters).collect()
    })
}

fn text(values: &Values, key: &str) -> Option<String> {
    values.get(key)?.try_clone().ok()?.try_into().ok()
}

fn number(values: &Values, key: &str) -> Option<u64> {
    let value = values.get(key)?;
    i64::try_from(value)
        .ok()
        .and_then(|number| u64::try_from(number).ok())
        .or_else(|| u64::try_from(value).ok())
}

fn artists(metadata: &Values) -> String {
    let list: Option<Vec<String>> = metadata
        .get("xesam:artist")
        .and_then(|value| value.try_clone().ok())
        .and_then(|value| value.try_into().ok());
    list.map_or_else(
        || text(metadata, "xesam:artist").unwrap_or_default(),
        |artists| artists.join(", "),
    )
}
