use crate::activity::Progress;
use serde::{Deserialize, Serialize};

const MINUTE_MS: u64 = 60_000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "mode")]
pub enum Timing {
    #[default]
    Off,
    Elapsed,
    Countdown {
        minutes: u32,
    },
}

impl Timing {
    pub const DEFAULT_COUNTDOWN: Self = Self::Countdown { minutes: 25 };

    pub fn progress(self, started_ms: u64) -> Progress {
        match self {
            Self::Off => Progress::None,
            Self::Elapsed => Progress::Since {
                start_ms: started_ms,
            },
            Self::Countdown { minutes } => Progress::Until {
                end_ms: started_ms.saturating_add(u64::from(minutes) * MINUTE_MS),
            },
        }
    }
}
