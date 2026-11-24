use crate::API;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const LONGEST_NAME: usize = 40;
const LONGEST_ABOUT: usize = 200;
const LOWEST_PORT: u16 = 1024;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api: u32,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub about: String,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)]
    pub translations: BTreeMap<String, Words>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Words {
    pub name: String,
    pub about: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Permissions {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hosts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub programs: bool,
}

impl Permissions {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

impl Manifest {
    pub fn parse(text: &str) -> Option<Self> {
        let manifest: Self = serde_json::from_str(text).ok()?;
        manifest.is_valid().then_some(manifest)
    }

    pub fn words(&self, language: &str) -> Words {
        let plain = Words {
            name: self.name.clone(),
            about: self.about.clone(),
        };
        self.translations
            .get(language)
            .filter(|words| !words.name.trim().is_empty())
            .cloned()
            .unwrap_or(plain)
    }

    fn is_valid(&self) -> bool {
        let is_named = !self.name.trim().is_empty() && self.name.chars().count() <= LONGEST_NAME;
        let has_port = self.permissions.port.is_none_or(|port| port >= LOWEST_PORT);
        is_plain_id(&self.id)
            && is_named
            && self.about.chars().count() <= LONGEST_ABOUT
            && self.api == API
            && has_port
            && self
                .permissions
                .hosts
                .iter()
                .all(|host| is_plain_host(host))
            && self
                .permissions
                .files
                .iter()
                .all(|file| is_fixed_path(file))
    }
}

pub fn is_plain_id(id: &str) -> bool {
    let has_length = (2..=32).contains(&id.len());
    let is_plain = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-';
    has_length && id.bytes().all(is_plain) && !id.starts_with('-')
}

fn is_plain_host(host: &str) -> bool {
    let is_plain =
        |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-:".contains(&byte);
    !host.is_empty() && host.bytes().all(is_plain)
}

fn is_fixed_path(path: &str) -> bool {
    let is_anchored = ["~/", "%APPDATA%/", "%LOCALAPPDATA%/", "/"]
        .iter()
        .any(|start| path.starts_with(start));
    let is_narrow = path.trim_end_matches('/').matches('/').count() >= 2;
    is_anchored && is_narrow && !path.split('/').any(|part| part == "..")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(permissions: &str) -> Option<Manifest> {
        Manifest::parse(&format!(
            r#"{{"id":"game","name":"Game","version":"1","api":1,"permissions":{permissions}}}"#
        ))
    }

    #[test]
    fn reads_what_a_plugin_asks_for() {
        let asked =
            manifest(r#"{"files":["~/.game/log.txt"],"hosts":["api.example.com"],"port":4100}"#);
        let permissions = asked.unwrap().permissions;
        assert_eq!(permissions.port, Some(4100));
        assert!(!permissions.programs && !permissions.is_empty());
        assert!(manifest("{}").unwrap().permissions.is_empty());
    }

    #[test]
    fn refuses_to_ask_for_too_much() {
        for permissions in [
            r#"{"files":["~/"]}"#,
            r#"{"files":["/"]}"#,
            r#"{"files":["~/.ssh/../.ssh/id"]}"#,
            r#"{"files":["relative/path/x"]}"#,
            r#"{"hosts":["evil.example/path"]}"#,
            r#"{"hosts":["*"]}"#,
            r#"{"port":80}"#,
            r#"{"everything":true}"#,
        ] {
            assert_eq!(manifest(permissions), None, "{permissions}");
        }
    }

    #[test]
    fn keeps_ids_safe_for_a_folder() {
        assert!(is_plain_id("game-dialogs2"));
        for id in ["", "a", "../x", "Game", "-x", "a/b", "a.b"] {
            assert!(!is_plain_id(id), "{id}");
        }
    }
}
