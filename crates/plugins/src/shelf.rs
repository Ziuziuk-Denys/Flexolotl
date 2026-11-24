use crate::{MANIFEST_FILE, Manifest, SCRIPT_FILE, manifest::is_plain_id};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

const LARGEST_FILE: u64 = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plugin {
    pub manifest: Manifest,
    pub script: String,
}

#[derive(Debug, Clone)]
pub struct Shelf {
    directory: PathBuf,
}

impl Shelf {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn plugins(&self) -> Vec<Plugin> {
        let Ok(entries) = fs::read_dir(&self.directory) else {
            return Vec::new();
        };
        let mut plugins: Vec<Plugin> = entries
            .flatten()
            .filter_map(|entry| self.load(entry.file_name().to_str()?))
            .collect();
        plugins.sort_by(|left, right| left.manifest.name.cmp(&right.manifest.name));
        plugins
    }

    pub fn load(&self, id: &str) -> Option<Plugin> {
        if !is_plain_id(id) {
            return None;
        }
        let folder = self.directory.join(id);
        let manifest = Manifest::parse(&read_text(&folder.join(MANIFEST_FILE))?)?;
        let script = read_text(&folder.join(SCRIPT_FILE))?;
        (manifest.id == id).then_some(Plugin { manifest, script })
    }

    pub fn install(&self, plugin: &Plugin, manifest_text: &str) -> io::Result<()> {
        let folder = self.directory.join(&plugin.manifest.id);
        fs::create_dir_all(&folder)?;
        fs::write(folder.join(SCRIPT_FILE), &plugin.script)?;
        fs::write(folder.join(MANIFEST_FILE), manifest_text)
    }

    pub fn remove(&self, id: &str) -> io::Result<()> {
        if !is_plain_id(id) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let folder = self.directory.join(id);
        for file in [MANIFEST_FILE, SCRIPT_FILE] {
            let _ = fs::remove_file(folder.join(file));
        }
        fs::remove_dir(folder)
    }
}

fn read_text(path: &Path) -> Option<String> {
    let size = fs::metadata(path).ok()?.len();
    (size <= LARGEST_FILE).then(|| fs::read_to_string(path).ok())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, process};

    #[test]
    fn installs_lists_and_removes() {
        let shelf = Shelf::new(env::temp_dir().join(format!("flexolotl-shelf-{}", process::id())));
        let text = r#"{"id":"clock","name":"Clock","version":"1","api":1}"#;
        let plugin = Plugin {
            manifest: Manifest::parse(text).unwrap(),
            script: "return {}".into(),
        };
        shelf.install(&plugin, text).unwrap();
        fs::create_dir_all(shelf.directory().join("broken")).unwrap();
        assert_eq!(shelf.plugins(), [plugin]);
        shelf.remove("clock").unwrap();
        assert!(shelf.plugins().is_empty() && shelf.remove("../clock").is_err());
        fs::remove_dir_all(shelf.directory()).unwrap();
    }

    #[test]
    fn needs_the_folder_to_carry_the_same_name() {
        let shelf = Shelf::new(env::temp_dir().join(format!("flexolotl-names-{}", process::id())));
        let text = r#"{"id":"clock","name":"Clock","version":"1","api":1}"#;
        let folder = shelf.directory().join("other");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join(MANIFEST_FILE), text).unwrap();
        fs::write(folder.join(SCRIPT_FILE), "return {}").unwrap();
        assert!(shelf.plugins().is_empty());
        fs::remove_dir_all(shelf.directory()).unwrap();
    }
}
