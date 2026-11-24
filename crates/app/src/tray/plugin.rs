use super::{done_button, quiet_link, remove_link};
use crate::{copy::Copy, shell::Shell, theme::Theme};
use flexolotl_plugins::Permissions;
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement, StatefulInteractiveElement, Styled, div,
    prelude::FluentBuilder, px,
};

pub fn render(shell: &Shell, id: &str, cx: &mut Context<Shell>) -> AnyElement {
    let copy = shell.copy();
    let Some(plugin) = shell.plugins.find(id) else {
        return div().into_any_element();
    };
    let manifest = &plugin.manifest;
    let is_enabled = shell.is_plugin_enabled(id);
    let (chosen, disabled, deleted) = (id.to_owned(), id.to_owned(), id.to_owned());
    let lines = describe(&manifest.permissions, copy);
    div()
        .flex()
        .flex_col()
        .child(paragraph(manifest.words(copy.code).about))
        .child(
            div()
                .mt(px(18.))
                .text_size(px(13.))
                .text_color(Theme::day().soft)
                .child(copy.plugin_may),
        )
        .children(lines.into_iter().map(|line| {
            div()
                .mt(px(6.))
                .text_size(px(16.))
                .line_height(px(22.))
                .child(line)
        }))
        .child(
            done_button(if is_enabled {
                copy.plugin_choose
            } else {
                copy.plugin_enable
            })
            .on_click(cx.listener(move |shell, _, window, cx| {
                if is_enabled {
                    shell.choose_plugin(chosen.clone(), window, cx);
                } else {
                    shell.enable_plugin(&chosen, window, cx);
                }
            })),
        )
        .when(is_enabled, |tray| {
            tray.child(quiet_link("disable", copy.plugin_disable).on_click(
                cx.listener(move |shell, _, window, cx| {
                    shell.disable_plugin(&disabled, window, cx)
                }),
            ))
        })
        .child(remove_link(copy.plugin_delete).on_click(
            cx.listener(move |shell, _, window, cx| shell.delete_plugin(&deleted, window, cx)),
        ))
        .into_any_element()
}

fn paragraph(text: String) -> impl IntoElement {
    div()
        .mt(px(8.))
        .line_height(px(21.))
        .text_color(Theme::day().soft)
        .child(text)
}

fn describe(permissions: &Permissions, copy: &Copy) -> Vec<String> {
    let mut lines = Vec::new();
    for file in &permissions.files {
        lines.push(format!("{} {file}", copy.may_read));
    }
    for host in &permissions.hosts {
        lines.push(format!("{} {host}", copy.may_visit));
    }
    if let Some(port) = permissions.port {
        lines.push(format!("{} {port}", copy.may_listen));
    }
    if permissions.programs {
        lines.push(copy.may_see_programs.to_owned());
    }
    if lines.is_empty() {
        lines.push(copy.may_nothing.to_owned());
    }
    lines
}
