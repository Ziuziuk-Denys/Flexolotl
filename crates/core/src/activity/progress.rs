use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "mode")]
pub enum Progress {
    #[default]
    None,
    Since {
        start_ms: u64,
    },
    Until {
        end_ms: u64,
    },
    Between {
        start_ms: u64,
        end_ms: u64,
    },
}

impl Progress {
    pub(super) fn bounds(self) -> (Option<u64>, Option<u64>) {
        match self {
            Self::None => (None, None),
            Self::Since { start_ms } => (Some(start_ms), None),
            Self::Until { end_ms } => (None, Some(end_ms)),
            Self::Between { start_ms, end_ms } => (Some(start_ms), Some(end_ms)),
        }
    }

    pub(super) fn is_backwards(self) -> bool {
        matches!(self, Self::Between { start_ms, end_ms } if end_ms <= start_ms)
    }
}
