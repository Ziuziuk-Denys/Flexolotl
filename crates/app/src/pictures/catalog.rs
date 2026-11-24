use crate::{platform::paths, shell::Shell};
use flexolotl_core::media::MediaKind;
use gpui_kit::{Context, ImageSource, SharedString, Window};
use std::{fs, io, path::PathBuf};

impl Shell {
    pub fn local_copy(&self, link: &str) -> ImageSource {
        let link = link.trim();
        match self.gallery.file_of(link) {
            Some(file) => pictures_directory().join(file).into(),
            None => SharedString::from(link.to_owned()).into(),
        }
    }

    pub fn remember_picture(&mut self, link: &str, file: Option<String>) {
        self.gallery.remember(link, file);
        self.saver.save_gallery(&self.gallery);
    }

    pub fn forget_picture(&mut self, link: &str, cx: &mut Context<Self>) {
        if let Some(file) = self.gallery.file_of(link) {
            let _ = fs::remove_file(pictures_directory().join(file));
        }
        self.gallery.forget(link);
        self.saver.save_gallery(&self.gallery);
        cx.notify();
    }

    pub fn use_picture(&mut self, link: String, window: &mut Window, cx: &mut Context<Self>) {
        self.remember_picture(&link, None);
        self.deck.current_mut().picture = link;
        self.close_tray(window, cx);
    }

    pub(crate) fn collect_pictures(&mut self) {
        let links: Vec<String> = self
            .deck
            .statuses()
            .iter()
            .map(|status| status.picture.trim().to_owned())
            .filter(|link| !link.is_empty() && !self.gallery.contains(link))
            .collect();
        for link in links {
            self.remember_picture(&link, None);
        }
    }
}

pub fn keep(link: &str, bytes: &[u8], kind: MediaKind) -> io::Result<String> {
    let name = link.rsplit('/').next().unwrap_or_default();
    let stem = name.split('.').next().unwrap_or_default();
    let is_plain = !stem.is_empty() && stem.bytes().all(|byte| byte.is_ascii_alphanumeric());
    if !is_plain {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let file = format!("{stem}.{}", kind.extension());
    fs::create_dir_all(pictures_directory())?;
    fs::write(pictures_directory().join(&file), bytes)?;
    Ok(file)
}

fn pictures_directory() -> PathBuf {
    paths::config_directory().join("pictures")
}
