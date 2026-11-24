use super::{issue::Problem, link::Link};
use serde::{Deserialize, Serialize};

const MAX_LENGTH: usize = 300;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Picture {
    pub source: String,
    pub caption: String,
    pub link: Link,
}

impl Picture {
    pub fn is_empty(&self) -> bool {
        self.source.trim().is_empty()
    }

    pub fn is_remote(&self) -> bool {
        self.source.contains("://")
    }

    pub fn is_loadable(&self) -> bool {
        self.is_remote() && self.source_problem().is_none()
    }

    pub(super) fn source_problem(&self) -> Option<Problem> {
        if self.source.len() > MAX_LENGTH {
            return Some(Problem::TooLong { limit: MAX_LENGTH });
        }
        if self.is_remote() {
            return Link::new(&*self.source).problem();
        }
        let is_asset_key = self
            .source
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'));
        (!is_asset_key).then_some(Problem::NotAPublicHttpsLink)
    }
}
