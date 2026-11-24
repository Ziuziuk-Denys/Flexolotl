mod lrc;
mod song;

pub use lrc::parse;
pub use song::Song;

const SHORTEST_LINE_MS: u64 = 4_000;
const LONGEST_LINE: usize = 128;
const SEPARATOR: &str = " / ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub start_ms: u64,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lyrics {
    lines: Vec<Line>,
}

impl Lyrics {
    pub fn from_lrc(text: &str) -> Self {
        Self {
            lines: join_short_lines(parse(text)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn line_at(&self, position_ms: u64) -> Option<&str> {
        let after = self
            .lines
            .partition_point(|line| line.start_ms <= position_ms);
        let line = &self.lines[after.checked_sub(1)?];
        (!line.text.is_empty()).then_some(line.text.as_str())
    }
}

fn join_short_lines(lines: Vec<Line>) -> Vec<Line> {
    let mut joined: Vec<Line> = Vec::with_capacity(lines.len());
    for line in lines {
        match joined.last_mut() {
            Some(last) if fits_together(last, &line) => {
                last.text.push_str(SEPARATOR);
                last.text.push_str(&line.text);
            }
            _ => joined.push(line),
        }
    }
    joined
}

fn fits_together(last: &Line, line: &Line) -> bool {
    let is_short = line.start_ms.saturating_sub(last.start_ms) < SHORTEST_LINE_MS;
    let length = last.text.chars().count() + SEPARATOR.len() + line.text.chars().count();
    let both_have_words = !last.text.is_empty() && !line.text.is_empty();
    is_short && both_have_words && length <= LONGEST_LINE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_lines_that_discord_would_skip() {
        let lyrics = Lyrics::from_lrc("[00:01.00]one\n[00:03.00]two\n[00:09.50]three");
        assert_eq!(lyrics.line_at(500), None);
        assert_eq!(lyrics.line_at(1_000), Some("one / two"));
        assert_eq!(lyrics.line_at(9_499), Some("one / two"));
        assert_eq!(lyrics.line_at(60_000), Some("three"));
    }

    #[test]
    fn is_silent_during_a_break() {
        let lyrics = Lyrics::from_lrc("[00:01.00]one\n[00:06.00]\n[00:20.00]two");
        assert_eq!(lyrics.line_at(10_000), None);
        assert_eq!(lyrics.line_at(21_000), Some("two"));
    }

    #[test]
    fn never_makes_a_line_too_long() {
        let long = "a".repeat(100);
        let text = format!("[00:01.00]{long}\n[00:02.00]{long}");
        let lyrics = Lyrics::from_lrc(&text);
        assert_eq!(lyrics.line_at(1_500), Some(long.as_str()));
    }
}
