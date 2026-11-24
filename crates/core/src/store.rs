use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Store {
    directory: PathBuf,
}

impl Store {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    pub fn load<T: DeserializeOwned>(&self, name: &str) -> Option<T> {
        let path = self.path(name);
        let text = fs::read_to_string(&path).ok()?;
        match serde_json::from_str(&text) {
            Ok(value) => Some(value),
            Err(_) => {
                let _ = fs::rename(&path, self.path(&format!("{name}.broken")));
                None
            }
        }
    }

    pub fn save<T: Serialize>(&self, name: &str, value: &T) -> io::Result<()> {
        fs::create_dir_all(&self.directory)?;
        let text = serde_json::to_string_pretty(value)?;
        write_atomically(&self.path(name), &text)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.directory.join(format!("{name}.json"))
    }
}

fn write_atomically(path: &Path, text: &str) -> io::Result<()> {
    let draft = path.with_extension("draft");
    fs::write(&draft, text)?;
    fs::rename(draft, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Deck;
    use std::{env, process};

    fn store(name: &str) -> Store {
        Store::new(env::temp_dir().join(format!("flexolotl-store-{}-{name}", process::id())))
    }

    #[test]
    fn reads_back_what_it_wrote() {
        let store = store("round-trip");
        let deck = Deck::starting_with("hello");
        store.save("deck", &deck).unwrap();
        assert_eq!(store.load::<Deck>("deck"), Some(deck));
        fs::remove_dir_all(&store.directory).unwrap();
    }

    #[test]
    fn keeps_a_broken_file_aside() {
        let store = store("broken");
        fs::create_dir_all(&store.directory).unwrap();
        fs::write(store.path("deck"), "{oops").unwrap();
        assert_eq!(store.load::<Deck>("deck"), None);
        assert_eq!(
            fs::read_to_string(store.path("deck.broken")).unwrap(),
            "{oops"
        );
        fs::remove_dir_all(&store.directory).unwrap();
    }
}
