use super::catalog::CatalogState;
use crate::{
    preferences::{heading, note},
    shell::{Screen, Shell},
    theme::Theme,
    tray::Tray,
};
use gpui_kit::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    Stateful, StatefulInteractiveElement, Styled, div, px,
};

const GUIDE: &str = "https://github.com/Ziuziuk-Denys/Flexolotl/tree/main/plugins";

impl Shell {
    pub fn open_plugins(&mut self, cx: &mut Context<Self>) {
        self.tray = None;
        self.plugins.reload();
        self.read_catalog(cx);
        self.go_to(Screen::Plugins, cx);
    }

    pub fn ask_about_plugin(&mut self, id: String, cx: &mut Context<Self>) {
        self.open_tray(Tray::Plugin(id), cx);
    }

    pub fn render_plugins(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::mixed(self.reveal().round());
        let copy = self.copy();
        div()
            .size_full()
            .pt(px(60.))
            .text_color(theme.ink)
            .child(
                div()
                    .id("plugins")
                    .size_full()
                    .overflow_y_scroll()
                    .px(px(40.))
                    .pb(px(40.))
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .pt(px(24.))
                            .text_size(px(28.))
                            .line_height(px(36.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(copy.plugins),
                    )
                    .child(note(copy.plugins_note, theme))
                    .children(self.installed_rows(theme, cx))
                    .children(self.catalog_rows(theme, cx))
                    .children(self.maker_rows(theme, cx)),
            )
            .into_any_element()
    }

    fn installed_rows(&self, theme: Theme, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let copy = self.copy();
        let installed = self.plugins.installed();
        if installed.is_empty() {
            return Vec::new();
        }
        let mut rows = vec![heading(copy.plugins_yours, theme)];
        rows.extend(installed.iter().map(|plugin| {
            let manifest = &plugin.manifest;
            let id = manifest.id.clone();
            let state = if self.is_plugin_enabled(&id) {
                copy.plugin_on
            } else {
                copy.plugin_off
            };
            let words = manifest.words(copy.code);
            row(
                &format!("mine {id}"),
                &words.name,
                &words.about,
                state,
                theme,
            )
            .on_click(cx.listener(move |shell, _, _, cx| {
                shell.ask_about_plugin(id.clone(), cx);
            }))
            .into_any_element()
        }));
        rows
    }

    fn catalog_rows(&self, theme: Theme, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let copy = self.copy();
        let catalog = &self.plugins.catalog;
        let mut rows = vec![heading(copy.plugins_catalog, theme)];
        match catalog.state {
            CatalogState::Unread | CatalogState::Reading => rows.push(note(copy.connecting, theme)),
            CatalogState::Failed => rows.push(note(copy.upload_failed, theme)),
            CatalogState::Read => {}
        }
        let available = catalog.entries.iter().filter(|entry| {
            let installed = self.plugins.find(&entry.id);
            installed.is_none_or(|plugin| plugin.manifest.version != entry.version)
        });
        rows.extend(available.map(|entry| {
            let id = entry.id.clone();
            let is_busy = catalog.installing.as_ref() == Some(&id);
            let action = match (is_busy, self.plugins.find(&id)) {
                (true, _) => "...",
                (false, Some(_)) => copy.restart_updated,
                (false, None) => copy.plugin_install,
            };
            let words = entry.words(copy.code);
            let about = format!("{} \u{b7} {}", words.about, entry.author);
            row(&format!("catalog {id}"), &words.name, &about, action, theme)
                .on_click(cx.listener(move |shell, _, _, cx| {
                    shell.install_plugin(id.clone(), cx);
                }))
                .into_any_element()
        }));
        rows
    }

    fn maker_rows(&self, theme: Theme, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let copy = self.copy();
        let directory = self.plugins.shelf.directory().to_owned();
        vec![
            heading(copy.plugins_make, theme),
            row("folder", copy.plugins_folder, "", "", theme)
                .on_click(cx.listener(move |_, _, _, cx| {
                    let _ = std::fs::create_dir_all(&directory);
                    cx.open_with_system(&directory);
                }))
                .into_any_element(),
            row("guide", copy.plugins_guide, "", "", theme)
                .on_click(cx.listener(|_, _, _, cx| cx.open_url(GUIDE)))
                .into_any_element(),
        ]
    }
}

fn row(
    id: &str,
    name: &str,
    about: &str,
    state: &'static str,
    theme: Theme,
) -> Stateful<gpui_kit::Div> {
    div()
        .id(SharedString::from(id.to_owned()))
        .flex_none()
        .py(px(14.))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(16.))
        .cursor_pointer()
        .border_b_1()
        .border_color(theme.hairline)
        .hover(|style| style.opacity(0.7))
        .child(
            div()
                .min_w_0()
                .flex()
                .flex_col()
                .child(div().truncate().child(name.to_owned()))
                .children((!about.is_empty()).then(|| {
                    div()
                        .mt(px(3.))
                        .text_size(px(13.))
                        .line_height(px(18.))
                        .text_color(theme.soft)
                        .child(about.to_owned())
                })),
        )
        .child(div().flex_none().text_color(theme.soft).child(state))
}
