mod catalog;
mod page;

use crate::{platform::paths, shell::Shell};
use catalog::Catalog;
use flexolotl_core::{activity::Activity, status::Source};
use flexolotl_plugins::{Event, Fetch, Manifest, Plugin, Runner, Shelf};
use futures::{AsyncReadExt, executor::block_on};
use gpui_kit::{Context, Task, Window};
use http_client::{AsyncBody, HttpClient};
use std::sync::Arc;

const LARGEST_ANSWER: u64 = 1024 * 1024;

pub struct Plugins {
    shelf: Shelf,
    installed: Vec<Plugin>,
    catalog: Catalog,
    running: Option<Running>,
}

struct Running {
    id: String,
    shown: Option<Box<Activity>>,
    problem: Option<String>,
    _runner: Runner,
    _listener: Task<()>,
}

impl Plugins {
    pub fn load() -> Self {
        let shelf = Shelf::new(paths::config_directory().join("plugins"));
        Self {
            installed: shelf.plugins(),
            shelf,
            catalog: Catalog::default(),
            running: None,
        }
    }

    pub fn installed(&self) -> &[Plugin] {
        &self.installed
    }

    pub fn find(&self, id: &str) -> Option<&Plugin> {
        self.installed
            .iter()
            .find(|plugin| plugin.manifest.id == id)
    }

    pub fn shown(&self) -> Option<&Activity> {
        self.running.as_ref()?.shown.as_deref()
    }

    pub fn problem(&self) -> Option<&str> {
        self.running.as_ref()?.problem.as_deref()
    }

    fn reload(&mut self) {
        self.installed = self.shelf.plugins();
    }
}

impl Shell {
    pub fn uses_plugin(&self) -> bool {
        self.deck.current().source == Source::Plugin
    }

    pub fn is_plugin_enabled(&self, id: &str) -> bool {
        let agreed = self.settings.plugins.get(id);
        let plugin = self.plugins.find(id);
        plugin.is_some_and(|plugin| agreed == Some(&asked_permissions(&plugin.manifest)))
    }

    pub fn enabled_plugins(&self) -> Vec<(String, String)> {
        self.plugins
            .installed()
            .iter()
            .map(|plugin| {
                let words = plugin.manifest.words(self.copy().code);
                (plugin.manifest.id.clone(), words.name)
            })
            .filter(|(id, _)| self.is_plugin_enabled(id))
            .collect()
    }

    pub fn plugin_activity(&self) -> Option<Activity> {
        let mut activity = self.plugins.shown()?.clone();
        let status = self.deck.current();
        if activity.large_picture.source.is_empty() {
            activity.large_picture.source = status.picture.trim().to_owned();
        }
        if activity.buttons.is_empty() {
            activity.buttons.clone_from(&status.buttons);
        }
        Some(activity)
    }

    pub fn choose_plugin(&mut self, id: String, window: &mut Window, cx: &mut Context<Self>) {
        let status = self.deck.current_mut();
        status.source = Source::Plugin;
        status.plugin = id;
        self.tune_sources(cx);
        self.close_tray(window, cx);
        self.go_home(cx);
    }

    pub fn enable_plugin(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(plugin) = self.plugins.find(id) else {
            return;
        };
        let agreed = asked_permissions(&plugin.manifest);
        self.settings.plugins.insert(id.to_owned(), agreed);
        self.settings_changed(window, cx);
        self.choose_plugin(id.to_owned(), window, cx);
    }

    pub fn disable_plugin(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.settings.plugins.remove(id);
        self.settings_changed(window, cx);
        self.tune_sources(cx);
        self.close_tray(window, cx);
    }

    pub fn delete_plugin(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let _ = self.plugins.shelf.remove(id);
        self.plugins.reload();
        self.disable_plugin(id, window, cx);
    }

    pub fn tune_sources(&mut self, cx: &mut Context<Self>) {
        self.tune_music(cx);
        self.tune_plugin(cx);
    }

    fn tune_plugin(&mut self, cx: &mut Context<Self>) {
        let wanted = self.deck.current().plugin.clone();
        let is_wanted = self.uses_plugin() && self.is_plugin_enabled(&wanted);
        let running = self.plugins.running.as_ref().map(|running| &running.id);
        if is_wanted && running == Some(&wanted) {
            return;
        }
        self.plugins.running = None;
        let Some(plugin) = self.plugins.find(&wanted).filter(|_| is_wanted).cloned() else {
            return;
        };
        let (runner, events) = Runner::start(plugin, fetch_through(cx.http_client()));
        let listener = cx.spawn(async move |this, cx| {
            while let Ok(event) = events.recv().await {
                let update = this.update(cx, |shell, cx| shell.hear_plugin(event, cx));
                if update.is_err() {
                    break;
                }
            }
        });
        self.plugins.running = Some(Running {
            id: wanted,
            shown: None,
            problem: None,
            _runner: runner,
            _listener: listener,
        });
    }

    fn hear_plugin(&mut self, event: Event, cx: &mut Context<Self>) {
        let Some(running) = &mut self.plugins.running else {
            return;
        };
        (running.shown, running.problem) = match event {
            Event::Shows(activity) => (activity, None),
            Event::Failed(problem) => (None, Some(problem)),
        };
        self.publish();
        cx.notify();
    }
}

fn asked_permissions(manifest: &Manifest) -> String {
    serde_json::to_string(&manifest.permissions).unwrap_or_default()
}

fn fetch_through(client: Arc<dyn HttpClient>) -> Fetch {
    Arc::new(move |url| block_on(fetch(&client, url, false)))
}

async fn fetch(client: &Arc<dyn HttpClient>, url: &str, follows: bool) -> Option<String> {
    let response = client.get(url, AsyncBody::empty(), follows).await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let mut text = String::new();
    response
        .into_body()
        .take(LARGEST_ANSWER)
        .read_to_string(&mut text)
        .await
        .ok()?;
    Some(text)
}
