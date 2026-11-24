const NOISE: [&str; 18] = [
    "official",
    "video",
    "audio",
    "lyric",
    "visuali",
    "music",
    "remaster",
    "explicit",
    "hd",
    "hq",
    "4k",
    "mv",
    "m/v",
    "live",
    "клип",
    "премьера",
    "текст",
    "слова",
];
const TOPIC: &str = " - Topic";
const DASHES: [&str; 3] = [" - ", " \u{2013} ", " \u{2014} "];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Song {
    pub artist: String,
    pub title: String,
}

impl Song {
    pub fn from_player(artist: &str, title: &str) -> Self {
        let artist = tidy_artist(artist);
        let title = without_noise(title);
        let split = DASHES
            .iter()
            .find_map(|dash| title.split_once(dash))
            .filter(|(left, right)| !left.trim().is_empty() && !right.trim().is_empty());
        match split {
            Some((left, right)) => Self {
                artist: left.trim().to_owned(),
                title: right.trim().to_owned(),
            },
            None => Self { artist, title },
        }
    }
}

fn tidy_artist(artist: &str) -> String {
    let artist = artist.trim();
    let artist = artist.strip_suffix(TOPIC).unwrap_or(artist);
    let artist = artist.strip_suffix("VEVO").unwrap_or(artist);
    artist.trim().to_owned()
}

fn without_noise(title: &str) -> String {
    let mut kept = String::with_capacity(title.len());
    let mut rest = title;
    while let Some(start) = rest.find(['(', '[']) {
        let close = if rest[start..].starts_with('(') {
            ')'
        } else {
            ']'
        };
        let Some(length) = rest[start..].find(close) else {
            break;
        };
        let group = rest[start..=start + length].to_lowercase();
        kept.push_str(&rest[..start]);
        if !NOISE.iter().any(|word| group.contains(word)) {
            kept.push_str(&rest[start..=start + length]);
        }
        rest = &rest[start + length + 1..];
    }
    kept.push_str(rest);
    kept.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn song(artist: &str, title: &str) -> (String, String) {
        let song = Song::from_player(artist, title);
        (song.artist, song.title)
    }

    fn expected(artist: &str, title: &str) -> (String, String) {
        (artist.to_owned(), title.to_owned())
    }

    #[test]
    fn reads_the_song_out_of_a_video_title() {
        assert_eq!(
            song(
                "RickAstleyVEVO",
                "Rick Astley - Never Gonna Give You Up (Official Video) [4K]"
            ),
            expected("Rick Astley", "Never Gonna Give You Up")
        );
        assert_eq!(
            song("Кино - Topic", "Группа крови"),
            expected("Кино", "Группа крови")
        );
    }

    #[test]
    fn keeps_brackets_that_belong_to_the_song() {
        assert_eq!(
            song("Artist", "Song (Part II)"),
            expected("Artist", "Song (Part II)")
        );
        assert_eq!(
            song("Artist", "Odd (title"),
            expected("Artist", "Odd (title")
        );
    }
}
