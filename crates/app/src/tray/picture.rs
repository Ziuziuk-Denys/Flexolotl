use super::{
    Tray, done_button, field_colors, labelled, outlined_button, problem, quiet_link, remove_link,
};
use crate::{
    pictures::{LocalPicture, LocalState, PictureProblem},
    shell::Shell,
    theme::Theme,
    widgets::{FieldEvent, TextField},
};
use flexolotl_core::activity::Picture;
use gpui_kit::{
    AnyElement, Context, IntoElement, ObjectFit, ParentElement, StatefulInteractiveElement, Styled,
    StyledImage, Window, div, img, prelude::FluentBuilder, px, relative,
};

const PREVIEW: f32 = 72.;

pub fn render(shell: &Shell, field: &TextField, cx: &mut Context<Shell>) -> AnyElement {
    match &shell.local_picture {
        Some(local) => chosen(shell, local, cx),
        None => choice(shell, field, cx),
    }
}

fn choice(shell: &Shell, field: &TextField, cx: &mut Context<Shell>) -> AnyElement {
    let copy = shell.copy();
    let text = field.text(cx);
    let is_bad = !text.trim().is_empty() && !is_usable(&text);
    let trouble = shell.picture_problem.map(|trouble| match trouble {
        PictureProblem::TooLarge => copy.picture_too_large,
        PictureProblem::WrongKind => copy.picture_wrong_kind,
    });
    div()
        .flex()
        .flex_col()
        .child(
            div().mt(px(16.)).child(
                outlined_button("choose-picture", copy.choose_picture)
                    .on_click(cx.listener(|shell, _, _, cx| shell.choose_picture(cx))),
            ),
        )
        .children(trouble.map(problem))
        .child(labelled(copy.picture_label, field))
        .when(is_bad, |tray| tray.child(problem(copy.needs_link)))
        .child(
            done_button(copy.done)
                .on_click(cx.listener(|shell, _, window, cx| shell.finish_picture(window, cx))),
        )
        .when(!shell.deck.current().picture.is_empty(), |tray| {
            tray.child(remove_link(copy.remove_picture).on_click(cx.listener(
                |shell, _, window, cx| {
                    shell.deck.current_mut().picture.clear();
                    shell.close_tray(window, cx);
                },
            )))
        })
        .into_any_element()
}

fn chosen(shell: &Shell, local: &LocalPicture, cx: &mut Context<Shell>) -> AnyElement {
    let copy = shell.copy();
    let is_busy = local.state == LocalState::Uploading;
    let percent = (local.share_sent() * 100.).round() as u32;
    let action = match local.state {
        LocalState::Chosen => copy.upload.to_owned(),
        LocalState::Uploading if percent < 100 => format!("{} {percent}%", copy.uploading),
        LocalState::Uploading => copy.upload_waiting.to_owned(),
        LocalState::Failed => copy.upload_again.to_owned(),
    };
    div()
        .flex()
        .flex_col()
        .child(
            div()
                .mt(px(16.))
                .flex()
                .items_center()
                .gap(px(16.))
                .child(
                    img(local.path.clone())
                        .size(px(PREVIEW))
                        .flex_none()
                        .rounded(px(12.))
                        .object_fit(ObjectFit::Cover),
                )
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_size(px(18.))
                        .child(local.name.clone()),
                ),
        )
        .child(
            div()
                .mt(px(14.))
                .line_height(px(21.))
                .text_color(Theme::day().soft)
                .child(copy.upload_note),
        )
        .when(local.state == LocalState::Failed, |tray| {
            tray.child(problem(copy.upload_failed))
        })
        .when(is_busy, |tray| tray.child(progress_bar(local.share_sent())))
        .child(
            done_button(action).on_click(cx.listener(move |shell, _, _, cx| {
                if !is_busy {
                    shell.upload_picture(cx);
                }
            })),
        )
        .when(!is_busy, |tray| {
            tray.child(
                quiet_link("choose-another", copy.choose_another)
                    .on_click(cx.listener(|shell, _, _, cx| shell.choose_picture(cx))),
            )
        })
        .into_any_element()
}

fn progress_bar(share: f32) -> impl IntoElement {
    let day = Theme::day();
    div()
        .mt(px(16.))
        .h(px(4.))
        .rounded_full()
        .bg(day.hairline)
        .child(div().h_full().w(relative(share)).rounded_full().bg(day.ink))
}

impl Shell {
    pub fn open_picture_tray(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let field = TextField::new(
            "https://",
            self.deck.current().picture.clone(),
            field_colors(),
            Self::edit_picture,
            window,
            cx,
        );
        field.focus(window, cx);
        self.open_tray(Tray::Picture(field), cx);
    }

    fn edit_picture(&mut self, event: FieldEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            FieldEvent::Submitted => self.finish_picture(window, cx),
            FieldEvent::Changed => cx.notify(),
            FieldEvent::Left => {}
        }
    }

    fn finish_picture(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Tray::Picture(field)) = &self.tray else {
            return;
        };
        let text = field.text(cx).trim().to_owned();
        if text.is_empty() || is_usable(&text) {
            self.deck.current_mut().picture = text;
            self.close_tray(window, cx);
        }
    }
}

fn is_usable(text: &str) -> bool {
    Picture {
        source: text.trim().to_owned(),
        ..Picture::default()
    }
    .is_loadable()
}
