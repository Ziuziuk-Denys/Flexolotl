use std::time::Instant;

const DRIFT_MS: u64 = 2_500;
const YOUTUBE_HOSTS: [&str; 4] = [
    "www.youtube.com",
    "youtube.com",
    "music.youtube.com",
    "m.youtube.com",
];

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub player: String,
    pub cover: Option<String>,
    pub is_from_browser: bool,
    pub length_ms: u64,
    position_ms: u64,
    seen: Instant,
}

impl Track {
    pub fn new(title: String, artist: String, player: String, length_ms: u64) -> Self {
        Self {
            title,
            artist,
            player,
            cover: None,
            is_from_browser: false,
            length_ms,
            position_ms: 0,
            seen: Instant::now(),
        }
    }

    pub fn heard_in_browser(mut self, is_from_browser: bool) -> Self {
        self.is_from_browser = is_from_browser;
        self
    }

    pub fn with_cover(mut self, cover: Option<String>) -> Self {
        self.cover = cover.filter(|url| url.starts_with("https://"));
        self
    }

    pub fn on_page(mut self, page: Option<&str>) -> Self {
        if let Some(id) = page.and_then(youtube_id) {
            self.player = "YouTube".to_owned();
            self.cover = Some(format!("https://i.ytimg.com/vi/{id}/hqdefault.jpg"));
        }
        self
    }

    pub fn at(mut self, position_ms: u64) -> Self {
        self.position_ms = position_ms;
        self.seen = Instant::now();
        self
    }

    pub fn key(&self) -> String {
        format!("{}\n{}", self.artist, self.title)
    }

    pub fn position_ms(&self) -> u64 {
        let passed = self.seen.elapsed().as_millis() as u64;
        let position = self.position_ms + passed;
        if self.length_ms > 0 {
            position.min(self.length_ms)
        } else {
            position
        }
    }

    pub fn continues(&self, earlier: &Self) -> bool {
        let is_same = self.key() == earlier.key()
            && self.cover == earlier.cover
            && self.player == earlier.player
            && self.length_ms == earlier.length_ms;
        is_same && self.position_ms().abs_diff(earlier.position_ms()) < DRIFT_MS
    }
}

fn youtube_id(page: &str) -> Option<&str> {
    let rest = page.strip_prefix("https://")?;
    let (host, path) = rest.split_once('/')?;
    let id = if host == "youtu.be" {
        path.split(['?', '&', '#']).next()?
    } else if YOUTUBE_HOSTS.contains(&host) {
        let query = path.split_once('?')?.1;
        query
            .split(['&', '#'])
            .find_map(|pair| pair.strip_prefix("v="))?
    } else {
        return None;
    };
    let is_plain = id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));
    (id.len() == 11 && is_plain).then_some(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(position_ms: u64) -> Track {
        Track::new("a".into(), "b".into(), "Nuvo".into(), 60_000).at(position_ms)
    }

    #[test]
    fn notices_a_seek_but_not_normal_playback() {
        assert!(track(10_000).continues(&track(9_500)));
        assert!(!track(40_000).continues(&track(9_500)));
    }

    #[test]
    fn keeps_only_public_covers() {
        let local = track(0).with_cover(Some("file:///home/me/cover.png".into()));
        assert_eq!(local.cover, None);
    }

    #[test]
    fn never_runs_past_the_end() {
        assert_eq!(track(90_000).position_ms(), 60_000);
    }

    #[test]
    fn takes_the_cover_from_a_youtube_page() {
        let page = "https://www.youtube.com/watch?list=x&v=dQw4w9WgXcQ&t=3";
        let track = track(0).on_page(Some(page));
        assert_eq!(track.player, "YouTube");
        assert_eq!(
            track.cover.as_deref(),
            Some("https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg")
        );
        for page in [
            "https://evil.example/watch?v=dQw4w9WgXcQ",
            "https://youtu.be/../../../..",
        ] {
            assert_eq!(youtube_id(page), None, "{page}");
        }
    }
}
