use super::issue::Problem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Link(String);

impl Link {
    pub const MAX_LENGTH: usize = 512;

    pub fn new(url: impl Into<String>) -> Self {
        Self(url.into().trim().to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn host(&self) -> Option<&str> {
        let rest = self.0.strip_prefix("https://")?;
        let authority = rest.split(['/', '?', '#']).next()?;
        (!authority.is_empty()).then_some(authority)
    }

    pub(super) fn problem(&self) -> Option<Problem> {
        if self.is_empty() {
            return None;
        }
        if self.0.len() > Self::MAX_LENGTH {
            return Some(Problem::TooLong {
                limit: Self::MAX_LENGTH,
            });
        }
        match self.host() {
            Some(host) if is_public_host(host) && !self.has_unsafe_characters() => None,
            _ => Some(Problem::NotAPublicHttpsLink),
        }
    }

    fn has_unsafe_characters(&self) -> bool {
        self.0
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
    }
}

fn is_public_host(host: &str) -> bool {
    let name = host.split(':').next().unwrap_or_default();
    let is_ip_address = name.chars().all(|c| c.is_ascii_digit() || c == '.');
    !host.contains(['@', '[']) && name.contains('.') && !is_ip_address
}
