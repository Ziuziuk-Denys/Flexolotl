use flexolotl_core::activity::{
    Activity, ActivityKind, Button, Link, MemberLine, Picture, Progress,
};
use serde::Deserialize;

const LONGEST_LINE: usize = 128;
const MOST_BUTTONS: usize = 2;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Shown {
    title: String,
    subtitle: String,
    kind: ActivityKind,
    picture: String,
    started: Option<f64>,
    ends: Option<f64>,
    buttons: Vec<ShownButton>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ShownButton {
    label: String,
    link: String,
}

impl Shown {
    pub fn into_activity(self, plugin_name: &str) -> Option<Activity> {
        let title = cut(&self.title, LONGEST_LINE);
        if title.is_empty() {
            return None;
        }
        let subtitle = cut(&self.subtitle, LONGEST_LINE);
        let has_bar = self.kind.shows_progress_bar();
        let (name, details, state) = if has_bar {
            (cut(plugin_name, LONGEST_LINE), title, subtitle)
        } else {
            (title, subtitle, String::new())
        };
        let picture = self.picture.trim();
        let activity = Activity {
            member_line: if details.is_empty() {
                MemberLine::Name
            } else {
                MemberLine::Details
            },
            name,
            details,
            state,
            kind: self.kind,
            large_picture: Picture {
                source: if picture.starts_with("https://") {
                    picture.to_owned()
                } else {
                    String::new()
                },
                ..Picture::default()
            },
            progress: progress(self.started, self.ends),
            buttons: self
                .buttons
                .into_iter()
                .map(|button| Button {
                    label: cut(&button.label, Button::MAX_LABEL),
                    link: Link::new(button.link),
                })
                .filter(|button| button.link.host().is_some() && !button.label.is_empty())
                .take(MOST_BUTTONS)
                .collect(),
            ..Activity::default()
        };
        activity.issues().is_empty().then_some(activity)
    }
}

fn progress(started: Option<f64>, ends: Option<f64>) -> Progress {
    match (started.map(milliseconds), ends.map(milliseconds)) {
        (Some(start_ms), Some(end_ms)) if end_ms > start_ms => {
            Progress::Between { start_ms, end_ms }
        }
        (Some(start_ms), _) => Progress::Since { start_ms },
        (None, Some(end_ms)) => Progress::Until { end_ms },
        (None, None) => Progress::None,
    }
}

fn milliseconds(seconds: f64) -> u64 {
    (seconds.max(0.) * 1000.) as u64
}

fn cut(text: &str, length: usize) -> String {
    text.trim().chars().take(length).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shown(json: &str) -> Option<Activity> {
        let shown: Shown = serde_json::from_str(json).unwrap();
        shown.into_activity("Game")
    }

    #[test]
    fn makes_a_status_from_what_a_plugin_says() {
        let activity = shown(
            r#"{"title":"Cave","subtitle":"hello there","started":10,
                "picture":"http://plain.example/a.png",
                "buttons":[{"label":"Page","link":"https://a.example"},{"label":"Bad","link":"file:///x"}]}"#,
        )
        .unwrap();
        assert_eq!(
            (&*activity.name, &*activity.details),
            ("Cave", "hello there")
        );
        assert_eq!(activity.progress, Progress::Since { start_ms: 10_000 });
        assert_eq!(activity.large_picture.source, "");
        assert_eq!(activity.buttons.len(), 1);
    }

    #[test]
    fn shows_nothing_without_a_title() {
        assert!(shown(r#"{"title":"  ","subtitle":"x"}"#).is_none());
    }

    #[test]
    fn cuts_what_discord_would_refuse() {
        let long = "x".repeat(500);
        let activity = shown(&format!(r#"{{"title":"{long}","kind":"listening"}}"#)).unwrap();
        assert_eq!(activity.details.len(), LONGEST_LINE);
        assert_eq!(activity.name, "Game");
    }
}
