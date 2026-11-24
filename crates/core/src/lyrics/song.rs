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
const SUFFIXES: [&str; 3] = [" - Topic", "VEVO", " Official"];
const GUESTS: [&str; 6] = [" feat", " ft.", " ft ", " prod", " при уч", " w/ "];
const JOINERS: [&str; 6] = [",", " & ", " x ", " X ", " и ", ";"];
const ALTERED: [&str; 8] = [
    "slowed",
    "sped",
    "speed",
    "reverb",
    "nightcore",
    "daycore",
    "ускорен",
    "замедлен",
];
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

    pub fn spellings(&self) -> Vec<Self> {
        let first = first_artist(&self.artist);
        let bare = bare_title(&self.title);
        let mut spellings = vec![self.clone()];
        for (artist, title) in [
            (first, self.title.as_str()),
            (self.artist.as_str(), bare.as_str()),
            (first, bare.as_str()),
        ] {
            let spelling = Self {
                artist: artist.to_owned(),
                title: title.to_owned(),
            };
            if !spellings.contains(&spelling) {
                spellings.push(spelling);
            }
        }
        spellings
    }

    pub fn has_changed_speed(&self) -> bool {
        let title = self.title.to_lowercase();
        ALTERED.iter().any(|word| title.contains(word))
    }
}

fn tidy_artist(artist: &str) -> String {
    let artist = artist.trim();
    let tidy = SUFFIXES
        .iter()
        .find_map(|suffix| artist.strip_suffix(suffix))
        .unwrap_or(artist);
    tidy.trim().to_owned()
}

fn first_artist(artist: &str) -> &str {
    let end = JOINERS
        .iter()
        .chain(&GUESTS)
        .filter_map(|joiner| artist.find(joiner))
        .min()
        .unwrap_or(artist.len());
    match artist[..end].trim() {
        "" => artist,
        first => first,
    }
}

fn bare_title(title: &str) -> String {
    let brackets = title.find(['(', '[']).unwrap_or(title.len());
    let guests = GUESTS
        .iter()
        .filter_map(|guest| title.find(guest))
        .min()
        .unwrap_or(title.len());
    match title[..brackets.min(guests)].trim() {
        "" => title.to_owned(),
        bare => bare.to_owned(),
    }
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
    fn offers_simpler_spellings() {
        let song = Song::from_player("Travis Scott", "Travis Scott - FE!N ft. Playboi Carti");
        let simple = Song {
            artist: "Travis Scott".into(),
            title: "FE!N".into(),
        };
        assert!(song.spellings().contains(&simple));
        let song = Song::from_player("Post Malone, Swae Lee", "Sunflower (Remix)");
        let simple = Song {
            artist: "Post Malone".into(),
            title: "Sunflower".into(),
        };
        assert_eq!(song.spellings().last(), Some(&simple));
    }

    #[test]
    fn knows_a_changed_speed() {
        assert!(Song::from_player("a", "b (Slowed + Reverb)").has_changed_speed());
        assert!(!Song::from_player("a", "b").has_changed_speed());
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
