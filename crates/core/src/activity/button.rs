use super::{issue::Problem, link::Link, text};
use serde::{Deserialize, Serialize};

pub const MAX_BUTTONS: usize = 2;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Button {
    pub label: String,
    pub link: Link,
}

impl Button {
    pub const MAX_LABEL: usize = 32;

    pub(super) fn label_problem(&self) -> Option<Problem> {
        match text::present(&self.label) {
            None => Some(Problem::Missing),
            Some(label) if label.chars().count() > Self::MAX_LABEL => Some(Problem::TooLong {
                limit: Self::MAX_LABEL,
            }),
            Some(_) => None,
        }
    }

    pub(super) fn link_problem(&self) -> Option<Problem> {
        if self.link.is_empty() {
            return Some(Problem::Missing);
        }
        self.link.problem()
    }
}
