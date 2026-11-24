use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    #[default]
    Playing,
    Listening,
    Watching,
    Competing,
}

impl ActivityKind {
    pub const ALL: [Self; 4] = [
        Self::Playing,
        Self::Listening,
        Self::Watching,
        Self::Competing,
    ];

    pub(super) fn code(self) -> u8 {
        match self {
            Self::Playing => 0,
            Self::Listening => 2,
            Self::Watching => 3,
            Self::Competing => 5,
        }
    }

    pub fn shows_progress_bar(self) -> bool {
        matches!(self, Self::Listening | Self::Watching)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberLine {
    #[default]
    Name,
    State,
    Details,
}

impl MemberLine {
    pub const ALL: [Self; 3] = [Self::Name, Self::State, Self::Details];

    pub(super) fn code(self) -> u8 {
        match self {
            Self::Name => 0,
            Self::State => 1,
            Self::Details => 2,
        }
    }
}
