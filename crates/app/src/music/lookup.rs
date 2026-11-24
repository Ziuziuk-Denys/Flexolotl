use super::Track;
use flexolotl_core::lyrics::{Lyrics, Song};
use futures::AsyncReadExt;
use http_client::{AsyncBody, HttpClient};
use serde_json::Value;
use std::{fmt::Write, sync::Arc};

const LYRICS_GET: &str = "https://lrclib.net/api/get";
const LYRICS_SEARCH: &str = "https://lrclib.net/api/search";
const COVERS: &str = "https://itunes.apple.com/search";
const MAX_RESPONSE: u64 = 2 * 1024 * 1024;
const LENGTH_TOLERANCE_SECONDS: f64 = 6.;

#[derive(Default)]
pub struct Found {
    pub lyrics: Option<Lyrics>,
    pub cover: Option<String>,
}

pub async fn find(client: Arc<dyn HttpClient>, track: &Track) -> Found {
    let song = Song::from_player(&track.artist, &track.title);
    let lyrics = find_lyrics(&client, &song, track.length_ms).await;
    let cover = match &track.cover {
        Some(_) => None,
        None => find_cover(&client, &song).await,
    };
    Found { lyrics, cover }
}

async fn find_lyrics(client: &Arc<dyn HttpClient>, song: &Song, length_ms: u64) -> Option<Lyrics> {
    let seconds = length_ms / 1000;
    let mut query = vec![
        ("artist_name", song.artist.clone()),
        ("track_name", song.title.clone()),
    ];
    if seconds > 0 {
        query.push(("duration", seconds.to_string()));
    }
    if let Some(answer) = get(client, LYRICS_GET, &query).await
        && let Some(lyrics) = synced(&answer)
    {
        return Some(lyrics);
    }
    let words = [("q", format!("{} {}", song.artist, song.title))];
    let answer = get(client, LYRICS_SEARCH, &words).await?;
    answer
        .as_array()?
        .iter()
        .filter(|entry| seconds == 0 || is_as_long(entry, seconds as f64))
        .find_map(synced)
}

fn is_as_long(entry: &Value, seconds: f64) -> bool {
    entry["duration"]
        .as_f64()
        .is_some_and(|length| (length - seconds).abs() <= LENGTH_TOLERANCE_SECONDS)
}

fn synced(entry: &Value) -> Option<Lyrics> {
    let lyrics = Lyrics::from_lrc(entry["syncedLyrics"].as_str()?);
    (!lyrics.is_empty()).then_some(lyrics)
}

async fn find_cover(client: &Arc<dyn HttpClient>, song: &Song) -> Option<String> {
    let query = [
        ("term", format!("{} {}", song.artist, song.title)),
        ("media", "music".to_owned()),
        ("entity", "song".to_owned()),
        ("limit", "5".to_owned()),
    ];
    let answer = get(client, COVERS, &query).await?;
    let title = song.title.to_lowercase();
    let result = answer["results"].as_array()?.iter().find(|result| {
        result["trackName"]
            .as_str()
            .is_some_and(|name| similar(&name.to_lowercase(), &title))
    })?;
    let small = result["artworkUrl100"].as_str()?;
    let large = small.replace("100x100bb", "512x512bb");
    large.starts_with("https://").then_some(large)
}

fn similar(left: &str, right: &str) -> bool {
    left.contains(right) || right.contains(left)
}

async fn get(
    client: &Arc<dyn HttpClient>,
    endpoint: &str,
    query: &[(&str, String)],
) -> Option<Value> {
    let mut url = endpoint.to_owned();
    for (index, (name, value)) in query.iter().enumerate() {
        let separator = if index == 0 { '?' } else { '&' };
        let _ = write!(url, "{separator}{name}={}", escape(value));
    }
    let response = client.get(&url, AsyncBody::empty(), true).await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let mut text = String::new();
    response
        .into_body()
        .take(MAX_RESPONSE)
        .read_to_string(&mut text)
        .await
        .ok()?;
    serde_json::from_str(&text).ok()
}

fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                escaped.push(char::from(byte));
            }
            _ => {
                let _ = write!(escaped, "%{byte:02X}");
            }
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::escape;

    #[test]
    fn escapes_everything_that_could_change_the_address() {
        assert_eq!(escape("a&b=c d"), "a%26b%3Dc%20d");
        assert_eq!(escape("ё"), "%D1%91");
    }
}
