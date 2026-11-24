mod catalog;
mod progress;
mod shrink;
mod upload;

use crate::{shell::Shell, tray::Tray};
use flexolotl_core::media::{self, MediaKind};
use gpui_kit::{Context, PathPromptOptions, SharedString, Task};
use progress::Progress;
use std::time::Duration;
use std::{
    fs,
    path::{Path, PathBuf},
};
use upload::Upload;

const LARGEST_FILE: u64 = 60 * 1024 * 1024;
const PROGRESS_TICK: Duration = Duration::from_millis(60);

pub struct LocalPicture {
    pub name: SharedString,
    pub path: PathBuf,
    pub state: LocalState,
    upload: Upload,
    progress: Progress,
    _tasks: Vec<Task<()>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalState {
    Chosen,
    Uploading,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PictureProblem {
    TooLarge,
    WrongKind,
}

impl LocalPicture {
    pub fn share_sent(&self) -> f32 {
        self.progress.share()
    }
}

impl Shell {
    pub fn choose_picture(&mut self, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: None,
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned());
            let source = path.clone();
            let read = cx
                .background_executor()
                .spawn(async move { read_picture(&source) })
                .await;
            let _ = this.update(cx, |shell, cx| {
                shell.picture_read(name.unwrap_or_default(), path, read, cx);
            });
        })
        .detach();
    }

    pub fn upload_picture(&mut self, cx: &mut Context<Self>) {
        let client = cx.http_client();
        let Some(local) = &mut self.local_picture else {
            return;
        };
        local.state = LocalState::Uploading;
        local.progress = Progress::of(local.upload.size());
        let (upload, progress) = (local.upload.clone(), local.progress.clone());
        let sending = cx.spawn(async move |this, cx| {
            let link = upload.clone().send(client, progress).await;
            let kept = link.map(|link| {
                let file = upload.keep(&link).ok();
                (link, file)
            });
            let _ = this.update(cx, |shell, cx| shell.picture_uploaded(kept, cx));
        });
        let watching = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(PROGRESS_TICK).await;
                let update = this.update(cx, |_, cx| cx.notify());
                if update.is_err() {
                    break;
                }
            }
        });
        local._tasks = vec![sending, watching];
        cx.notify();
    }

    fn picture_read(
        &mut self,
        name: String,
        path: PathBuf,
        read: Result<Upload, PictureProblem>,
        cx: &mut Context<Self>,
    ) {
        if !matches!(self.tray, Some(Tray::Picture(_))) {
            return;
        }
        match read {
            Ok(upload) => {
                self.picture_problem = None;
                self.local_picture = Some(LocalPicture {
                    name: name.into(),
                    path,
                    state: LocalState::Chosen,
                    upload,
                    progress: Progress::default(),
                    _tasks: Vec::new(),
                });
            }
            Err(problem) => {
                self.local_picture = None;
                self.picture_problem = Some(problem);
            }
        }
        cx.notify();
    }

    fn picture_uploaded(&mut self, kept: Option<(String, Option<String>)>, cx: &mut Context<Self>) {
        match kept {
            Some((link, file)) => {
                self.remember_picture(&link, file);
                self.deck.current_mut().picture = link;
                self.local_picture = None;
                self.tray = None;
                self.status_changed(cx);
            }
            None => {
                if let Some(local) = &mut self.local_picture {
                    local.state = LocalState::Failed;
                    local._tasks.clear();
                }
            }
        }
        cx.notify();
    }
}

fn read_picture(path: &Path) -> Result<Upload, PictureProblem> {
    let size = fs::metadata(path).map_or(0, |data| data.len());
    if size > LARGEST_FILE {
        return Err(PictureProblem::TooLarge);
    }
    let bytes = fs::read(path).map_err(|_| PictureProblem::WrongKind)?;
    let kind = MediaKind::of(&bytes).ok_or(PictureProblem::WrongKind)?;
    let (bytes, kind) = shrink::prepare(&bytes, kind).ok_or(PictureProblem::WrongKind)?;
    if bytes.len() > media::MAX_BYTES {
        return Err(PictureProblem::TooLarge);
    }
    Ok(Upload::new(bytes, kind))
}
