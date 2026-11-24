use crate::shell::Shell;
use flexolotl_core::activity::{Activity, ActivityKind, MemberLine, Picture, Progress};

const LONGEST: usize = 128;

impl Shell {
    pub fn music_activity(&self, now_ms: u64) -> Option<Activity> {
        let track = self.music.track.as_ref()?;
        let status = self.deck.current();
        let credit = match track.artist.trim() {
            "" => track.title.clone(),
            artist => format!("{artist} - {}", track.title),
        };
        let (details, state) = match self.music.line() {
            Some(line) => (line.to_owned(), credit),
            None => (track.title.clone(), track.artist.clone()),
        };
        let start_ms = now_ms.saturating_sub(track.position_ms());
        let picture = match status.picture.trim() {
            "" => self.music.cover().unwrap_or_default().to_owned(),
            own => own.to_owned(),
        };
        Some(Activity {
            name: fit(&track.player),
            kind: if self.music.is_video() {
                ActivityKind::Watching
            } else {
                ActivityKind::Listening
            },
            member_line: MemberLine::Details,
            details: fit(&details),
            state: fit(&state),
            large_picture: Picture {
                source: picture,
                ..Picture::default()
            },
            progress: if track.length_ms > 0 {
                Progress::Between {
                    start_ms,
                    end_ms: start_ms + track.length_ms,
                }
            } else {
                Progress::Since { start_ms }
            },
            buttons: status.buttons.clone(),
            ..Activity::default()
        })
    }
}

fn fit(text: &str) -> String {
    let text: String = text.trim().chars().take(LONGEST).collect();
    if text.chars().count() == 1 {
        format!("{text}\u{2060}")
    } else {
        text
    }
}
