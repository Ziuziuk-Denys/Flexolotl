use serde::{Deserialize, Serialize};

const MOST: usize = 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedPicture {
    pub link: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Gallery {
    pictures: Vec<SavedPicture>,
}

impl Gallery {
    pub fn pictures(&self) -> &[SavedPicture] {
        &self.pictures
    }

    pub fn file_of(&self, link: &str) -> Option<&str> {
        self.find(link)?.file.as_deref()
    }

    pub fn contains(&self, link: &str) -> bool {
        self.find(link).is_some()
    }

    pub fn remember(&mut self, link: &str, file: Option<String>) {
        let link = link.trim();
        if link.is_empty() {
            return;
        }
        let known = self.find(link).and_then(|picture| picture.file.clone());
        self.forget(link);
        self.pictures.insert(
            0,
            SavedPicture {
                link: link.to_owned(),
                file: file.or(known),
            },
        );
        self.pictures.truncate(MOST);
    }

    pub fn forget(&mut self, link: &str) {
        self.pictures.retain(|picture| picture.link != link);
    }

    fn find(&self, link: &str) -> Option<&SavedPicture> {
        self.pictures
            .iter()
            .find(|picture| picture.link == link.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn puts_the_latest_first_without_repeating() {
        let mut gallery = Gallery::default();
        gallery.remember("https://a/1.png", Some("1.png".into()));
        gallery.remember("https://a/2.png", None);
        gallery.remember(" https://a/1.png ", None);
        let links: Vec<_> = gallery.pictures().iter().map(|p| p.link.as_str()).collect();
        assert_eq!(links, ["https://a/1.png", "https://a/2.png"]);
        assert_eq!(gallery.file_of("https://a/1.png"), Some("1.png"));
        gallery.remember("  ", None);
        gallery.forget("https://a/2.png");
        assert_eq!(gallery.pictures().len(), 1);
    }

    #[test]
    fn does_not_grow_without_end() {
        let mut gallery = Gallery::default();
        for number in 0..200 {
            gallery.remember(&format!("https://a/{number}.png"), None);
        }
        assert_eq!(gallery.pictures().len(), MOST);
        assert_eq!(gallery.pictures()[0].link, "https://a/199.png");
    }
}
