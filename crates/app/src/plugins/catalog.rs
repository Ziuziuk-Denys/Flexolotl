use super::fetch;
use crate::shell::Shell;
use flexolotl_plugins::{MANIFEST_FILE, Manifest, Plugin, SCRIPT_FILE, Words};
use gpui_kit::{Context, Task};
use http_client::HttpClient;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::Arc};

const ROOT: &str = "https://raw.githubusercontent.com/Ziuziuk-Denys/Flexolotl/main/plugins";

#[derive(Default)]
pub struct Catalog {
    pub entries: Vec<Entry>,
    pub state: CatalogState,
    pub installing: Option<String>,
    work: Option<Task<()>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CatalogState {
    #[default]
    Unread,
    Reading,
    Read,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub about: String,
    #[serde(default)]
    pub author: String,
    pub version: String,
    #[serde(default)]
    pub translations: BTreeMap<String, Words>,
}

impl Entry {
    pub fn words(&self, language: &str) -> Words {
        self.translations.get(language).cloned().unwrap_or(Words {
            name: self.name.clone(),
            about: self.about.clone(),
        })
    }
}

impl Shell {
    pub fn read_catalog(&mut self, cx: &mut Context<Self>) {
        let catalog = &mut self.plugins.catalog;
        if matches!(catalog.state, CatalogState::Reading | CatalogState::Read) {
            return;
        }
        catalog.state = CatalogState::Reading;
        let client = cx.http_client();
        catalog.work = Some(cx.spawn(async move |this, cx| {
            let entries = read_index(&client).await;
            let _ = this.update(cx, |shell, cx| {
                let catalog = &mut shell.plugins.catalog;
                catalog.state = match entries {
                    Some(_) => CatalogState::Read,
                    None => CatalogState::Failed,
                };
                catalog.entries = entries.unwrap_or_default();
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub fn install_plugin(&mut self, id: String, cx: &mut Context<Self>) {
        let client = cx.http_client();
        self.plugins.catalog.installing = Some(id.clone());
        self.plugins.catalog.work = Some(cx.spawn(async move |this, cx| {
            let found = download(&client, &id).await;
            let _ = this.update(cx, |shell, cx| {
                shell.plugins.catalog.installing = None;
                if let Some((plugin, manifest)) = found {
                    let _ = shell.plugins.shelf.install(&plugin, &manifest);
                    shell.plugins.reload();
                    shell.ask_about_plugin(id, cx);
                }
                cx.notify();
            });
        }));
        cx.notify();
    }
}

async fn read_index(client: &Arc<dyn HttpClient>) -> Option<Vec<Entry>> {
    let text = fetch(client, &format!("{ROOT}/index.json"), true).await?;
    serde_json::from_str(&text).ok()
}

async fn download(client: &Arc<dyn HttpClient>, id: &str) -> Option<(Plugin, String)> {
    let text = fetch(client, &format!("{ROOT}/{id}/{MANIFEST_FILE}"), true).await?;
    let manifest = Manifest::parse(&text).filter(|manifest| manifest.id == id)?;
    let script = fetch(client, &format!("{ROOT}/{id}/{SCRIPT_FILE}"), true).await?;
    Some((Plugin { manifest, script }, text))
}
