use crate::{copy::Copy, shell::now_ms};
use flexolotl_core::activity::Progress;

pub fn describe(progress: Progress, copy: &Copy) -> Option<String> {
    let now = now_ms();
    match progress {
        Progress::None => None,
        Progress::Since { start_ms } => Some(clock(now.saturating_sub(start_ms))),
        Progress::Until { end_ms } | Progress::Between { end_ms, .. } => {
            let left = clock(end_ms.saturating_sub(now));
            Some(format!("{} {left}", copy.left))
        }
    }
}

pub fn clock(milliseconds: u64) -> String {
    let seconds = milliseconds / 1000;
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::clock;

    #[test]
    fn adds_hours_only_when_needed() {
        assert_eq!(clock(5_000), "0:05");
        assert_eq!(clock(754_000), "12:34");
        assert_eq!(clock(3_661_000), "1:01:01");
    }
}
