use super::{Track, video};
use flexolotl_core::lyrics::{Lyrics, Song};
use futures::{
    AsyncReadExt,
    future::{join, join_all},
};
use http_client::{AsyncBody, HttpClient, Method, Request, Response};
use serde_json::Value;
use std::{fmt::Write, ops::RangeInclusive, sync::Arc};

const LYRICS: &str = "https://lrclib.net/api/search";
const COVERS: &str = "https://itunes.apple.com/search";
const MAX_RESPONSE: u64 = 2 * 1024 * 1024;
const LENGTH_TOLERANCE_SECONDS: f64 = 15.;
const SPEEDS: RangeInclusive<f64> = 0.6..=1.7;

pub async fn find_lyrics(client: Arc<dyn HttpClient>, track: Track) -> Option<Lyrics> {
    let song = Song::from_player(&track.artist, &track.title);
    let seconds = track.length_ms as f64 / 1000.;
    let spellings = song.spellings();
    let (exact, simpler) = spellings.split_first()?;
    if let Some(lyrics) = try_spelling(&client, &song, exact, seconds).await {
        return Some(lyrics);
    }
    let tries = simpler
        .iter()
        .map(|spelling| try_spelling(&client, &song, spelling, seconds));
    let words = [("q", format!("{} {}", song.artist, song.title))];
    let (found, loose) = join(join_all(tries), get(&client, LYRICS, &words)).await;
    found
        .into_iter()
        .flatten()
        .next()
        .or_else(|| best_lyrics(&loose?, &song, seconds, song.has_changed_speed()))
}

async fn try_spelling(
    client: &Arc<dyn HttpClient>,
    song: &Song,
    spelling: &Song,
    seconds: f64,
) -> Option<Lyrics> {
    let query = [
        ("artist_name", spelling.artist.clone()),
        ("track_name", spelling.title.clone()),
    ];
    let answer = get(client, LYRICS, &query).await?;
    best_lyrics(&answer, spelling, seconds, song.has_changed_speed())
}

fn best_lyrics(
    answer: &Value,
    song: &Song,
    seconds: f64,
    has_changed_speed: bool,
) -> Option<Lyrics> {
    let distance = |entry: &Value| {
        let length = entry["duration"].as_f64().unwrap_or_default();
        (length - seconds).abs()
    };
    let entry = answer
        .as_array()?
        .iter()
        .filter(|entry| entry["syncedLyrics"].is_string())
        .filter(|entry| is_same(&entry["trackName"], &song.title))
        .min_by(|left, right| distance(left).total_cmp(&distance(right)))?;
    let text = entry["syncedLyrics"].as_str()?;
    let length = entry["duration"].as_f64()?;
    let ratio = seconds / length;
    let lyrics = if seconds <= 0. || distance(entry) <= LENGTH_TOLERANCE_SECONDS {
        Lyrics::from_lrc(text)
    } else if has_changed_speed && SPEEDS.contains(&ratio) {
        Lyrics::stretched(text, ratio)
    } else {
        return None;
    };
    (!lyrics.is_empty()).then_some(lyrics)
}

pub async fn find_cover(client: Arc<dyn HttpClient>, track: Track) -> Option<String> {
    let song = Song::from_player(&track.artist, &track.title);
    if !track.is_from_browser {
        return find_album_cover(&client, &song).await;
    }
    let (video, album) = join(
        find_video_cover(&client, &track),
        find_album_cover(&client, &song),
    )
    .await;
    video.or(album)
}

async fn find_video_cover(client: &Arc<dyn HttpClient>, track: &Track) -> Option<String> {
    let question = video::question(&format!("{} {}", track.title, track.artist));
    let request = Request::builder()
        .method(Method::POST)
        .uri(video::SEARCH)
        .header("Content-Type", "application/json")
        .body(AsyncBody::from(question))
        .ok()?;
    let answer = read(client.send(request).await.ok()?).await?;
    video::cover(&serde_json::from_str(&answer).ok()?, &track.title)
}

async fn find_album_cover(client: &Arc<dyn HttpClient>, song: &Song) -> Option<String> {
    let query = [
        ("term", format!("{} {}", song.artist, song.title)),
        ("media", "music".to_owned()),
        ("entity", "song".to_owned()),
        ("limit", "5".to_owned()),
    ];
    let answer = get(client, COVERS, &query).await?;
    let result = answer["results"].as_array()?.iter().find(|result| {
        is_same(&result["trackName"], &song.title) && is_same(&result["artistName"], &song.artist)
    })?;
    let small = result["artworkUrl100"].as_str()?;
    let large = small.replace("100x100bb", "512x512bb");
    large.starts_with("https://").then_some(large)
}

fn is_same(found: &Value, wanted: &str) -> bool {
    let found = found.as_str().unwrap_or_default().to_lowercase();
    let wanted = wanted.to_lowercase();
    !found.is_empty() && !wanted.is_empty() && (found.contains(&wanted) || wanted.contains(&found))
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
    serde_json::from_str(&read(response).await?).ok()
}

async fn read(response: Response<AsyncBody>) -> Option<String> {
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
    Some(text)
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
    use super::*;
    use serde_json::json;

    fn answer() -> Value {
        json!([
            {"trackName": "Song", "duration": 400, "syncedLyrics": "[00:01.00]far"},
            {"trackName": "Song", "duration": 200, "syncedLyrics": "[00:10.00]near"},
            {"trackName": "Song", "duration": 201, "syncedLyrics": null},
            {"trackName": "Other", "duration": 203, "syncedLyrics": "[00:01.00]wrong"}
        ])
    }

    #[test]
    fn takes_the_closest_length_and_stretches_only_a_slowed_song() {
        let song = Song::from_player("a", "Song");
        let found = best_lyrics(&answer(), &song, 205., false).unwrap();
        assert_eq!(found.line_at(11_000), Some("near"));
        assert_eq!(best_lyrics(&answer(), &song, 250., false), None);
        let slowed = best_lyrics(&answer(), &song, 250., true).unwrap();
        assert_eq!(slowed.line_at(11_000), None);
        assert_eq!(slowed.line_at(12_500), Some("near"));
    }

    #[test]
    fn needs_both_names_to_agree() {
        assert!(is_same(&json!("Rick Astley"), "rick astley"));
        assert!(!is_same(&json!("Simon & Garfunkel"), "jawed"));
        assert!(!is_same(&json!(null), "jawed"));
    }

    #[test]
    fn escapes_everything_that_could_change_the_address() {
        assert_eq!(escape("a&b=c d"), "a%26b%3Dc%20d");
        assert_eq!(escape("ё"), "%D1%91");
    }
}
