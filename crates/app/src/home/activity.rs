use super::{palette, time};
use crate::{
    shell::{Shell, now_ms},
    tray::Tray,
    widgets::{TextField, face},
};
use flexolotl_core::activity::{ActivityKind, MAX_BUTTONS, Picture, Progress};
use gpui_kit::{
    Animation, AnimationExt, AnyElement, Context, Div, FontWeight, InteractiveElement, IntoElement,
    ObjectFit, ParentElement, SharedString, Stateful, StatefulInteractiveElement, Styled,
    StyledImage, Window, div, ease_out_quint, img, prelude::FluentBuilder, px, relative,
};
use std::time::Duration;

const PICTURE: f32 = 60.;

impl Shell {
    pub(super) fn render_activity(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let lines = self.render_lines(window, cx);
        div()
            .mt(px(12.))
            .p(px(12.))
            .rounded(px(8.))
            .bg(palette::card())
            .flex()
            .flex_col()
            .child(
                self.render_verb()
                    .on_click(cx.listener(|shell, _, _, cx| shell.open_tray(Tray::Verb, cx))),
            )
            .child(
                div()
                    .mt(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        self.render_picture()
                            .on_click(cx.listener(|shell, _, window, cx| {
                                shell.open_picture_tray(window, cx);
                            })),
                    )
                    .child(lines),
            )
            .children(self.render_track_bar())
            .children(self.render_buttons(cx))
            .into_any_element()
    }

    fn render_verb(&self) -> Stateful<Div> {
        let kind = self.deck.current().kind;
        let copy = self.copy();
        let text = if self.uses_plugin() {
            self.plugin_verb()
        } else if self.uses_music() {
            let verb = if self.music.is_video() {
                ActivityKind::Watching
            } else {
                ActivityKind::Listening
            };
            let track = self.music.track.as_ref();
            let player = track.map_or("...", |track| &track.player);
            format!("{} {player}", copy.kind_verbs[verb as usize])
        } else if kind.shows_progress_bar() {
            let title = self.deck.current().title.trim();
            let title = if title.is_empty() { "Flexolotl" } else { title };
            format!("{} {title}", copy.kind_verbs[kind as usize])
        } else {
            copy.kinds[kind as usize].to_owned()
        };
        div()
            .id("verb")
            .cursor_pointer()
            .truncate()
            .text_size(px(12.))
            .line_height(px(16.))
            .font_weight(FontWeight::SEMIBOLD)
            .hover(|style| style.opacity(0.7))
            .child(text)
    }

    fn render_picture(&self) -> Stateful<Div> {
        let picture = Picture {
            source: self.shown_picture(),
            ..Picture::default()
        };
        div()
            .id("picture")
            .size(px(PICTURE))
            .flex_none()
            .rounded(px(8.))
            .overflow_hidden()
            .cursor_pointer()
            .bg(palette::artwork())
            .flex()
            .items_center()
            .justify_center()
            .hover(|style| style.opacity(0.8))
            .map(|frame| {
                if picture.is_loadable() {
                    frame.child(
                        img(self.local_copy(&picture.source))
                            .size(px(PICTURE))
                            .rounded(px(8.))
                            .object_fit(ObjectFit::Cover),
                    )
                } else {
                    let ink = palette::text().into();
                    frame.child(face(46., self.is_live, ink, gpui_kit::rgb(0xff8fa3).into()))
                }
            })
    }

    fn render_lines(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        if self.uses_music() {
            return self.render_track_lines();
        }
        if self.uses_plugin() {
            return self.render_plugin_lines();
        }
        let progress = self
            .activity()
            .map_or(Progress::None, |activity| activity.progress);
        let time = time::describe(progress, self.copy());
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .text_size(px(14.))
            .child(
                editable(&self.editor().title, "title", window, cx)
                    .font_weight(FontWeight::SEMIBOLD),
            )
            .child(editable(&self.editor().subtitle, "subtitle", window, cx))
            .child(
                div()
                    .id("time")
                    .mt(px(2.))
                    .cursor_pointer()
                    .text_size(px(12.))
                    .line_height(px(16.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(if time.is_some() {
                        palette::positive()
                    } else {
                        palette::hint()
                    })
                    .hover(|style| style.opacity(0.7))
                    .child(time.unwrap_or_else(|| self.copy().add_time.to_owned()))
                    .on_click(cx.listener(|shell, _, _, cx| shell.open_tray(Tray::Time, cx))),
            )
            .into_any_element()
    }

    fn plugin_verb(&self) -> String {
        let copy = self.copy();
        match self.plugins.shown() {
            Some(activity) if activity.kind.shows_progress_bar() => {
                format!(
                    "{} {}",
                    copy.kind_verbs[activity.kind as usize], activity.name
                )
            }
            Some(activity) => copy.kinds[activity.kind as usize].to_owned(),
            None => {
                let plugin = self.plugins.find(&self.deck.current().plugin);
                plugin.map_or(copy.plugins.to_owned(), |plugin| {
                    plugin.manifest.words(copy.code).name
                })
            }
        }
    }

    fn render_plugin_lines(&self) -> AnyElement {
        let activity = self.plugins.shown();
        let quiet = match self.plugins.problem() {
            Some(problem) => problem.to_owned(),
            None => self.copy().plugin_silent.to_owned(),
        };
        let (title, subtitle) = match activity {
            Some(activity) if activity.kind.shows_progress_bar() => {
                (activity.details.clone(), activity.state.clone())
            }
            Some(activity) => (activity.name.clone(), activity.details.clone()),
            None => (quiet, String::new()),
        };
        let progress = activity.map_or(Progress::None, |activity| activity.progress);
        let time = time::describe(progress, self.copy());
        spoken_lines(title, subtitle)
            .children(time.map(|time| {
                div()
                    .text_size(px(12.))
                    .line_height(px(16.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette::positive())
                    .child(time)
            }))
            .into_any_element()
    }

    fn shown_picture(&self) -> String {
        if self.uses_plugin() {
            let activity = self.plugin_activity();
            return activity.map_or_else(
                || self.deck.current().picture.clone(),
                |activity| activity.large_picture.source,
            );
        }
        let cover = self.music.cover().filter(|_| self.uses_music());
        cover.unwrap_or(&self.deck.current().picture).to_owned()
    }

    fn render_track_lines(&self) -> AnyElement {
        let activity = self.music_activity(now_ms());
        let title = activity
            .as_ref()
            .map_or(self.copy().nothing_plays, |activity| &activity.details);
        let credit = activity.as_ref().map_or("", |activity| &activity.state);
        spoken_lines(title.to_owned(), credit.to_owned()).into_any_element()
    }

    fn render_track_bar(&self) -> Option<AnyElement> {
        let track = self.music.track.as_ref().filter(|_| self.uses_music())?;
        if track.length_ms == 0 {
            return None;
        }
        let passed = track.position_ms();
        let share = passed as f32 / track.length_ms as f32;
        let label = |time| {
            div()
                .text_size(px(12.))
                .text_color(palette::text_soft())
                .child(time::clock(time))
        };
        let bar = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .mt(px(10.))
            .child(label(passed))
            .child(
                div()
                    .flex_1()
                    .h(px(2.))
                    .rounded_full()
                    .bg(palette::underline())
                    .child(
                        div()
                            .h_full()
                            .w(relative(share))
                            .rounded_full()
                            .bg(palette::text()),
                    ),
            )
            .child(label(track.length_ms));
        Some(bar.into_any_element())
    }

    fn render_buttons(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let copy = self.copy();
        let buttons = &self.deck.current().buttons;
        let mut rows: Vec<AnyElement> = buttons
            .iter()
            .enumerate()
            .map(|(index, button)| {
                let label = match button.label.trim() {
                    "" => copy.button,
                    label => label,
                };
                button_row(index, label.to_owned())
                    .on_click(cx.listener(move |shell, _, window, cx| {
                        shell.open_button_tray(index, window, cx);
                    }))
                    .into_any_element()
            })
            .collect();
        if buttons.len() < MAX_BUTTONS {
            let text = if buttons.is_empty() {
                copy.add_button
            } else {
                copy.add_second_button
            };
            rows.push(
                div()
                    .id("add-button")
                    .mt(px(6.))
                    .h(px(30.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .text_size(px(13.))
                    .text_color(palette::hint())
                    .hover(|style| style.text_color(palette::text()))
                    .child(text)
                    .on_click(cx.listener(|shell, _, window, cx| shell.add_button(window, cx)))
                    .into_any_element(),
            );
        }
        rows
    }
}

fn spoken_lines(title: String, subtitle: String) -> Div {
    let key = SharedString::from(title.clone());
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .text_size(px(14.))
        .line_height(px(19.))
        .child(
            div()
                .truncate()
                .font_weight(FontWeight::SEMIBOLD)
                .child(title)
                .with_animation(
                    key,
                    Animation::new(Duration::from_millis(260)).with_easing(ease_out_quint()),
                    |line, phase| line.opacity(phase),
                ),
        )
        .child(div().truncate().child(subtitle))
}

fn editable(
    field: &TextField,
    id: &'static str,
    window: &Window,
    cx: &Context<Shell>,
) -> Stateful<Div> {
    let is_focused = field.is_focused(window, cx);
    let line = if is_focused {
        gpui_kit::rgba(0)
    } else {
        palette::underline()
    };
    div()
        .id(id)
        .h(px(20.))
        .line_height(px(18.))
        .border_b_1()
        .border_dashed()
        .border_color(line)
        .when(!is_focused, |line| {
            line.hover(|style| style.border_color(palette::underline_hover()))
        })
        .child(field.element())
}

fn button_row(index: usize, label: String) -> Stateful<Div> {
    div()
        .id(("button", index))
        .mt(px(8.))
        .h(px(32.))
        .px(px(12.))
        .rounded(px(8.))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .bg(palette::button())
        .hover(|style| style.bg(palette::button_hover()))
        .text_size(px(14.))
        .font_weight(FontWeight::MEDIUM)
        .child(div().truncate().child(label))
}
