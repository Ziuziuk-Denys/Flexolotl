use super::is_browser;
use crate::music::Track;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as SessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
};

const TICKS_PER_MS: i64 = 10_000;

pub struct Media {
    manager: SessionManager,
}

impl Media {
    pub fn connect() -> Option<Self> {
        let manager = SessionManager::RequestAsync().ok()?.get().ok()?;
        Some(Self { manager })
    }

    pub fn playing(&self, hears_browsers: bool) -> Option<Track> {
        let mut tracks: Vec<Track> = self
            .manager
            .GetSessions()
            .ok()?
            .into_iter()
            .filter_map(|session| read_track(&session))
            .filter(|track| hears_browsers || !track.is_from_browser)
            .collect();
        tracks.sort_by_key(|track| track.is_from_browser);
        tracks.into_iter().next()
    }
}

fn read_track(session: &Session) -> Option<Track> {
    let status = session.GetPlaybackInfo().ok()?.PlaybackStatus().ok()?;
    if status != PlaybackStatus::Playing {
        return None;
    }
    let details = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
    let title = details.Title().ok()?.to_string();
    if title.trim().is_empty() {
        return None;
    }
    let artist = details.Artist().map(|artist| artist.to_string());
    let program = session.SourceAppUserModelId().ok()?.to_string();
    let (position_ms, length_ms) = timeline(session).unwrap_or((0, 0));
    let track = Track::new(
        title,
        artist.unwrap_or_default(),
        player_name(&program),
        length_ms,
    )
    .heard_in_browser(is_browser(short_name(&program)))
    .at(position_ms);
    Some(track)
}

fn timeline(session: &Session) -> Option<(u64, u64)> {
    let timeline = session.GetTimelineProperties().ok()?;
    let position = timeline.Position().ok()?.Duration / TICKS_PER_MS;
    let length = timeline.EndTime().ok()?.Duration / TICKS_PER_MS;
    Some((position.try_into().ok()?, length.try_into().ok()?))
}

fn short_name(program: &str) -> &str {
    let file = program.rsplit(['\\', '/']).next().unwrap_or(program);
    file.split('!').next().unwrap_or(file)
}

fn player_name(program: &str) -> String {
    let name = short_name(program);
    let name = name.strip_suffix(".exe").unwrap_or(name);
    let mut letters = name.chars();
    letters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(letters).collect()
    })
}
