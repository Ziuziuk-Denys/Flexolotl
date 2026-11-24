use super::{LEFT, TRACK_TOP};
use crate::{
    art::{Engraving, engraving},
    assets::icon,
    shell::Shell,
    theme::Theme,
    widgets::icon,
};
use flexolotl_core::activity::Picture;
use gpui_kit::{
    AnyElement, Context, Div, FontWeight, InteractiveElement, IntoElement, MouseButton, ObjectFit,
    ParentElement, SharedString, Stateful, StatefulInteractiveElement, Styled, StyledImage, div,
    img, prelude::FluentBuilder, px,
};

pub const COVER: f32 = 300.;
const RADIUS: f32 = 16.;
const TILTS: [f32; 5] = [0.2, -0.45, 0.9, -1.2, 0.];

impl Shell {
    pub(super) fn render_cover(
        &self,
        index: usize,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let copy = self.copy();
        let status = &self.deck.statuses()[index];
        let title = match status.title.trim() {
            "" => copy.untitled,
            title => title,
        };
        let can_delete = self.deck.statuses().len() > 1;
        self.page_at(index, theme)
            .child(self.render_artwork(index))
            .child(caption(title.to_owned(), status.subtitle.clone(), theme))
            .child(
                div()
                    .mt(px(20.))
                    .flex()
                    .gap(px(24.))
                    .text_color(theme.soft)
                    .child(
                        action(("copy", index), self.copy_label(index)).on_click(
                            cx.listener(move |shell, _, _, cx| shell.copy_status(index, cx)),
                        ),
                    )
                    .when(can_delete, |actions| {
                        actions.child(action(("delete", index), copy.delete).on_click(cx.listener(
                            move |shell, _, window, cx| {
                                shell.delete_status(index, window, cx);
                            },
                        )))
                    }),
            )
            .into_any_element()
    }

    pub(super) fn render_new_cover(
        &self,
        index: usize,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let copy = self.copy();
        let paste = self
            .notice
            .clone()
            .filter(|_| self.carousel.centred() == index)
            .unwrap_or_else(|| copy.paste_code.into());
        self.page_at(index, theme)
            .child(
                div()
                    .size(px(COVER))
                    .rounded(px(RADIUS))
                    .border_1()
                    .border_color(theme.hairline)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon(icon::PLUS, 20.).text_color(theme.ink)),
            )
            .child(caption(copy.new_status.to_owned(), String::new(), theme))
            .child(
                div().mt(px(20.)).text_color(theme.soft).child(
                    action(("paste", 0), paste).on_click(
                        cx.listener(|shell, _, window, cx| shell.paste_status(window, cx)),
                    ),
                ),
            )
            .into_any_element()
    }

    fn page_at(&self, index: usize, theme: Theme) -> Div {
        let left = LEFT + self.carousel.left_of(index);
        div()
            .absolute()
            .top(px(TRACK_TOP))
            .left(px(left))
            .w(px(COVER))
            .flex()
            .flex_col()
            .text_color(theme.ink)
    }

    fn render_artwork(&self, index: usize) -> impl IntoElement {
        let picture = Picture {
            source: self.deck.statuses()[index].picture.clone(),
            ..Picture::default()
        };
        div().relative().size(px(COVER)).map(|frame| {
            if picture.is_loadable() {
                frame.child(
                    img(picture.source.trim().to_owned())
                        .size(px(COVER))
                        .rounded(px(RADIUS))
                        .object_fit(ObjectFit::Cover),
                )
            } else {
                frame.child(engraving(Engraving {
                    portrait: self.portrait.image.clone(),
                    lines: self.settings.lines,
                    reveal: self.reveal().round(),
                    tilt: TILTS[index % TILTS.len()],
                    horizon: 0.,
                }))
            }
        })
    }

    fn copy_label(&self, index: usize) -> SharedString {
        self.notice
            .clone()
            .filter(|_| self.carousel.centred() == index)
            .unwrap_or_else(|| self.copy().copy_code.into())
    }
}

fn caption(title: String, subtitle: String, theme: Theme) -> impl IntoElement {
    div()
        .mt(px(24.))
        .h(px(56.))
        .flex()
        .flex_col()
        .child(
            div()
                .truncate()
                .text_size(px(22.))
                .line_height(px(30.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
        .child(
            div()
                .truncate()
                .text_size(px(16.))
                .line_height(px(24.))
                .text_color(theme.soft)
                .child(subtitle),
        )
}

fn action(id: (&'static str, usize), text: impl Into<SharedString>) -> Stateful<Div> {
    div()
        .id(id)
        .cursor_pointer()
        .hover(|style| style.opacity(0.6))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(text.into())
}
