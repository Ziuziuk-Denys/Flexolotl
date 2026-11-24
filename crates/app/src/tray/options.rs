use super::kind_icon;
use crate::{assets::icon as icon_path, shell::Shell, theme::Theme, widgets::icon};
use flexolotl_core::{activity::ActivityKind, status::Timing};
use gpui_kit::{
    AnyElement, Context, Div, InteractiveElement, IntoElement, ParentElement, SharedString,
    Stateful, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px, rgba,
};

const FIRST_PLUGIN: usize = 100;
const MORE_PLUGINS: usize = 99;

pub fn verbs(shell: &Shell, cx: &mut Context<Shell>) -> AnyElement {
    let copy = shell.copy();
    let is_text = !shell.uses_music() && !shell.uses_plugin();
    let current = is_text.then(|| shell.deck.current().kind);
    let plugins =
        shell
            .enabled_plugins()
            .into_iter()
            .enumerate()
            .map(|(index, (id, name))| {
                let is_current = shell.uses_plugin() && shell.deck.current().plugin == id;
                option(
                    FIRST_PLUGIN + index,
                    Some(icon_path::PLUGIN),
                    name,
                    is_current,
                )
                .on_click(cx.listener(move |shell, _, window, cx| {
                    shell.choose_plugin(id.clone(), window, cx)
                }))
            });
    let plugins: Vec<_> = plugins.collect();
    let rows = ActivityKind::ALL.map(|kind| {
        let label = copy.kinds[kind as usize];
        option(
            kind as usize,
            Some(kind_icon(kind)),
            label,
            current == Some(kind),
        )
        .on_click(cx.listener(move |shell, _, window, cx| {
            shell.choose_kind(kind, window, cx);
        }))
    });
    div()
        .mt(px(8.))
        .flex()
        .flex_col()
        .child(
            div()
                .mb(px(8.))
                .line_height(px(21.))
                .text_color(Theme::day().soft)
                .child(copy.verb_note),
        )
        .children(rows)
        .child(
            option(
                ActivityKind::ALL.len(),
                Some(icon_path::MUSIC),
                copy.music_source,
                shell.uses_music(),
            )
            .on_click(cx.listener(|shell, _, window, cx| {
                shell.choose_music(window, cx);
            })),
        )
        .children(plugins)
        .child(
            option(
                MORE_PLUGINS,
                Some(icon_path::PLUGIN),
                copy.plugins_more,
                false,
            )
            .on_click(cx.listener(|shell, _, _, cx| shell.open_plugins(cx))),
        )
        .into_any_element()
}

pub fn times(shell: &Shell, cx: &mut Context<Shell>) -> AnyElement {
    let copy = shell.copy();
    let current = shell.deck.current().timing;
    let choices = [Timing::Off, Timing::Elapsed, Timing::DEFAULT_COUNTDOWN];
    let rows = choices.into_iter().enumerate().map(|(index, timing)| {
        option(index, None, copy.time_options[index], timing == current).on_click(cx.listener(
            move |shell, _, window, cx| {
                shell.set_timing(timing, cx);
                shell.close_tray(window, cx);
            },
        ))
    });
    div()
        .mt(px(8.))
        .flex()
        .flex_col()
        .children(rows.collect::<Vec<_>>())
        .into_any_element()
}

fn option(
    index: usize,
    picture: Option<&'static str>,
    label: impl Into<SharedString>,
    is_current: bool,
) -> Stateful<Div> {
    let day = Theme::day();
    div()
        .id(("option", index))
        .h(px(54.))
        .flex()
        .items_center()
        .gap(px(14.))
        .cursor_pointer()
        .border_b_1()
        .border_color(rgba(0x1414121a))
        .text_size(px(18.))
        .hover(|style| style.opacity(0.65))
        .children(picture.map(|path| icon(path, 22.).text_color(day.ink)))
        .child(div().flex_1().child(label.into()))
        .when(is_current, |row| {
            row.child(div().size(px(8.)).rounded_full().bg(day.ink))
        })
}
