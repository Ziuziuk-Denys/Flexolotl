use flexolotl_core::lyrics::Lyrics;
use std::collections::HashMap;

const MOST: usize = 200;

#[derive(Debug)]
pub enum Search<T> {
    Running,
    Missing,
    Found(T),
}

impl<T> Search<T> {
    pub fn found(&self) -> Option<&T> {
        match self {
            Self::Found(value) => Some(value),
            Self::Running | Self::Missing => None,
        }
    }

    fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl<T> From<Option<T>> for Search<T> {
    fn from(value: Option<T>) -> Self {
        value.map_or(Self::Missing, Self::Found)
    }
}

#[derive(Debug)]
pub struct Find {
    pub lyrics: Search<Lyrics>,
    pub cover: Search<String>,
}

impl Find {
    pub fn is_complete(&self) -> bool {
        !self.lyrics.is_running() && !self.cover.is_running()
    }
}

#[derive(Debug, Default)]
pub struct Finds(HashMap<String, Find>);

impl Finds {
    pub fn get(&self, key: &str) -> Option<&Find> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Find> {
        self.0.get_mut(key)
    }

    pub fn start(&mut self, key: String, needs_cover: bool) {
        if self.0.len() >= MOST {
            self.0.clear();
        }
        let cover = if needs_cover {
            Search::Running
        } else {
            Search::Missing
        };
        let find = Find {
            lyrics: Search::Running,
            cover,
        };
        self.0.insert(key, find);
    }

    pub fn drop_unfinished(&mut self) {
        self.0.retain(|_, find| find.is_complete());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forgets_only_what_was_not_finished() {
        let mut finds = Finds::default();
        finds.start("a".into(), true);
        finds.start("b".into(), false);
        finds.get_mut("b").unwrap().lyrics = None.into();
        finds.drop_unfinished();
        assert!(finds.get("a").is_none());
        assert!(finds.get("b").unwrap().is_complete());
    }
}
