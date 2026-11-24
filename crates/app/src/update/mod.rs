mod download;
mod manifest;
mod target;
mod version;

use manifest::Package;
pub use version::Version;

use crate::{arguments::Arguments, background, shell::Shell};
use gpui_kit::{Context, Task};
use http_client::HttpClient;
use std::{
    env, fs,
    process::Command,
    sync::Arc,
    time::{Duration, Instant},
};
use target::Target;

const BETWEEN_CHECKS: Duration = Duration::from_secs(6 * 60 * 60);
const SHORTEST_BREAK: Duration = Duration::from_secs(10 * 60);

#[derive(Default)]
pub struct Update {
    pub state: UpdateState,
    target: Option<Target>,
    checked: Option<Instant>,
    work: Option<Task<()>>,
    clock: Option<Task<()>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum UpdateState {
    #[default]
    Idle,
    Ready,
    Elsewhere,
}

impl Shell {
    pub fn start_updates(&mut self, cx: &mut Context<Self>) {
        self.update.target = Target::find();
        if let Some(target) = &self.update.target {
            target.clean_up();
        }
        self.update.clock = Some(cx.spawn(async move |this, cx| {
            loop {
                let update = this.update(cx, |shell, cx| shell.check_for_update(cx));
                if update.is_err() {
                    break;
                }
                cx.background_executor().timer(BETWEEN_CHECKS).await;
            }
        }));
    }

    pub fn check_for_update(&mut self, cx: &mut Context<Self>) {
        let is_recent = self
            .update
            .checked
            .is_some_and(|checked| checked.elapsed() < SHORTEST_BREAK);
        if !self.settings.updates || is_recent || self.update.state != UpdateState::Idle {
            return;
        }
        let Some(target) = self.update.target.clone() else {
            return;
        };
        self.update.checked = Some(Instant::now());
        let client = cx.http_client();
        let finding = cx
            .background_executor()
            .spawn(async move { bring(client, target).await });
        self.update.work = Some(cx.spawn(async move |this, cx| {
            let state = finding.await;
            let _ = this.update(cx, |shell, cx| {
                shell.update.state = state.unwrap_or_default();
                shell.update.work = None;
                cx.notify();
            });
        }));
    }

    pub fn use_update(&mut self, cx: &mut Context<Self>) {
        match self.update.state {
            UpdateState::Idle => {}
            UpdateState::Ready => self.restart_updated(cx),
            UpdateState::Elsewhere => cx.open_url(&format!("{}/latest", download::RELEASES)),
        }
    }

    pub fn start_heir(&self) -> bool {
        let can_leave = cfg!(not(target_os = "macos")) && self.update.work.is_none();
        can_leave
            && self.start_next(Arguments {
                starts_hidden: true,
                is_heir: true,
                ..Arguments::default()
            })
    }

    fn restart_updated(&mut self, cx: &mut Context<Self>) {
        let arguments = Arguments {
            is_after_update: true,
            ..Arguments::default()
        };
        if self.start_next(arguments) {
            background::quit_later(cx);
        }
    }

    fn start_next(&self, arguments: Arguments) -> bool {
        let installed = self
            .update
            .target
            .as_ref()
            .map(|target| target.launched.clone());
        let Some(program) = installed.or_else(|| env::current_exe().ok()) else {
            return false;
        };
        let mut next = Command::new(program);
        let arguments = Arguments {
            live_since: self.is_live.then_some(self.started_ms),
            ..arguments
        };
        arguments.apply(&mut next);
        next.spawn().is_ok()
    }
}

async fn bring(client: Arc<dyn HttpClient>, target: Target) -> Option<UpdateState> {
    let package = download::newer_package(&client, &target.suffix).await?;
    if !target.is_replaceable() {
        return Some(UpdateState::Elsewhere);
    }
    let path = target.download_path();
    let fetched = download::fetch(&client, &package, &path).await;
    let installed = fetched.and_then(|()| target.install(&path).ok());
    let _ = fs::remove_file(path);
    installed.map(|()| UpdateState::Ready)
}
