mod deck;
mod source;
mod timing;

pub use deck::Deck;
pub use source::Source;
pub use timing::Timing;

use crate::activity::{Activity, ActivityKind, Button, MemberLine, Picture};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Status {
    pub title: String,
    pub subtitle: String,
    pub kind: ActivityKind,
    pub source: Source,
    pub picture: String,
    pub timing: Timing,
    pub buttons: Vec<Button>,
}

impl Status {
    pub fn activity(&self, started_ms: u64, member_line: MemberLine) -> Activity {
        let subtitle_line = if self.kind.shows_progress_bar() {
            MemberLine::State
        } else {
            MemberLine::Details
        };
        let has_subtitle = !self.subtitle.trim().is_empty();
        let mut activity = Activity {
            name: self.title.clone(),
            kind: self.kind,
            member_line: match member_line {
                MemberLine::Name => MemberLine::Name,
                _ if has_subtitle => subtitle_line,
                _ => MemberLine::Name,
            },
            large_picture: Picture {
                source: self.picture.trim().to_owned(),
                ..Picture::default()
            },
            progress: self.timing.progress(started_ms),
            buttons: self.buttons.clone(),
            ..Activity::default()
        };
        if self.kind.shows_progress_bar() {
            activity.details = self.title.clone();
            activity.state = self.subtitle.clone();
        } else {
            activity.details = self.subtitle.clone();
        }
        activity
    }

    pub fn is_blank(&self) -> bool {
        *self == Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(kind: ActivityKind, subtitle: &str) -> Status {
        Status {
            title: "title".into(),
            subtitle: subtitle.into(),
            kind,
            ..Status::default()
        }
    }

    #[test]
    fn puts_both_lines_where_discord_shows_them() {
        let playing = status(ActivityKind::Playing, "below").activity(0, MemberLine::Details);
        assert_eq!((&*playing.name, &*playing.details), ("title", "below"));
        assert_eq!(playing.member_line, MemberLine::Details);
        let watching = status(ActivityKind::Watching, "below").activity(0, MemberLine::Details);
        assert_eq!((&*watching.details, &*watching.state), ("title", "below"));
        assert_eq!(watching.member_line, MemberLine::State);
    }

    #[test]
    fn falls_back_to_the_title_next_to_the_name() {
        let activity = status(ActivityKind::Playing, " ").activity(0, MemberLine::Details);
        assert_eq!(activity.member_line, MemberLine::Name);
    }
}
