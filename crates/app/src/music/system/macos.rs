use crate::music::Track;
use std::process::Command;

const SEPARATOR: char = '\u{1f}';
const SCRIPT: &str = r#"
on report(player, title, artist, seconds, duration, cover)
    set separator to character id 31
    return player & separator & title & separator & artist & separator & seconds & separator & duration & separator & cover
end report
if application "Spotify" is running then
    tell application "Spotify"
        if player state is playing then
            set song to current track
            return my report("Spotify", name of song, artist of song, player position, (duration of song) / 1000, artwork url of song)
        end if
    end tell
end if
if application "Music" is running then
    tell application "Music"
        if player state is playing then
            set song to current track
            return my report("Apple Music", name of song, artist of song, player position, duration of song, "")
        end if
    end tell
end if
return ""
"#;

pub struct Media;

impl Media {
    pub fn connect() -> Option<Self> {
        Some(Self)
    }

    pub fn playing(&self, _hears_browsers: bool) -> Option<Track> {
        let output = Command::new("osascript")
            .args(["-e", SCRIPT])
            .output()
            .ok()?;
        let answer = String::from_utf8(output.stdout).ok()?;
        let mut parts = answer.trim_end().split(SEPARATOR).map(str::to_owned);
        let player = parts.next().filter(|player| !player.is_empty())?;
        let (title, artist) = (parts.next()?, parts.next()?);
        let (position, length) = (milliseconds(&parts.next()?), milliseconds(&parts.next()?));
        let track = Track::new(title, artist, player, length)
            .with_cover(parts.next())
            .at(position);
        Some(track)
    }
}

fn milliseconds(seconds: &str) -> u64 {
    seconds
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .map_or(0, |seconds| (seconds * 1000.) as u64)
}
