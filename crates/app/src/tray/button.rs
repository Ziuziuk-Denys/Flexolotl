use super::{Tray, done_button, field_colors, labelled, problem, remove_link};
use crate::{
    copy::Copy,
    shell::Shell,
    widgets::{FieldEvent, TextField},
};
use flexolotl_core::activity::{Activity, Button, Field, Link};
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement, StatefulInteractiveElement, Styled, Window,
    div, prelude::FluentBuilder,
};

pub struct ButtonTray {
    pub index: usize,
    pub is_new: bool,
    label: TextField,
    link: TextField,
    shows_problems: bool,
    has_bad_label: bool,
    has_bad_link: bool,
}

impl ButtonTray {
    pub fn render(&self, copy: &'static Copy, cx: &mut Context<Shell>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .child(labelled(copy.button_label, &self.label))
            .when(self.shows_problems && self.has_bad_label, |tray| {
                tray.child(problem(copy.needs_text))
            })
            .child(labelled(copy.button_link, &self.link))
            .when(self.shows_problems && self.has_bad_link, |tray| {
                tray.child(problem(copy.needs_link))
            })
            .child(
                done_button(copy.done)
                    .on_click(cx.listener(|shell, _, window, cx| shell.finish_button(window, cx))),
            )
            .when(!self.is_new, |tray| {
                tray.child(
                    remove_link(copy.remove_button).on_click(
                        cx.listener(|shell, _, window, cx| shell.remove_button(window, cx)),
                    ),
                )
            })
            .into_any_element()
    }

    fn button(&self, cx: &Context<Shell>) -> Button {
        Button {
            label: self.label.text(cx).trim().to_owned(),
            link: Link::new(self.link.text(cx)),
        }
    }

    fn check(&mut self, button: &Button) -> bool {
        let activity = Activity {
            buttons: vec![button.clone()],
            ..Activity::default()
        };
        let issues = activity.issues();
        let has = |field| issues.iter().any(|issue| issue.field == field);
        self.has_bad_label = has(Field::ButtonLabel(0));
        self.has_bad_link = has(Field::ButtonLink(0));
        issues.is_empty()
    }
}

impl Shell {
    pub fn add_button(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.deck.current_mut().buttons.push(Button::default());
        let index = self.deck.current().buttons.len() - 1;
        self.open_button(index, true, window, cx);
    }

    pub fn open_button_tray(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.open_button(index, false, window, cx);
    }

    fn open_button(
        &mut self,
        index: usize,
        is_new: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let copy = self.copy();
        let button = self.deck.current().buttons[index].clone();
        let label = TextField::new(
            copy.button_label_hint,
            button.label,
            field_colors(),
            Self::edit_button,
            window,
            cx,
        );
        let link = TextField::new(
            "https://",
            button.link.as_str().to_owned(),
            field_colors(),
            Self::edit_button,
            window,
            cx,
        );
        label.focus(window, cx);
        let tray = ButtonTray {
            index,
            is_new,
            label,
            link,
            shows_problems: false,
            has_bad_label: false,
            has_bad_link: false,
        };
        self.open_tray(Tray::Button(tray), cx);
    }

    fn edit_button(&mut self, event: FieldEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            FieldEvent::Submitted => self.submit_button_field(window, cx),
            FieldEvent::Changed => self.recheck_button(cx),
            FieldEvent::Left => {}
        }
    }

    fn submit_button_field(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match &self.tray {
            Some(Tray::Button(tray)) if tray.label.is_focused(window, cx) => {
                tray.link.focus(window, cx);
            }
            _ => self.finish_button(window, cx),
        }
    }

    fn recheck_button(&mut self, cx: &mut Context<Self>) {
        if let Some(Tray::Button(tray)) = &mut self.tray {
            let button = tray.button(cx);
            tray.check(&button);
            cx.notify();
        }
    }

    fn finish_button(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Tray::Button(tray)) = &mut self.tray else {
            return;
        };
        let button = tray.button(cx);
        if !tray.check(&button) {
            tray.shows_problems = true;
            return cx.notify();
        }
        tray.is_new = false;
        let index = tray.index;
        self.deck.current_mut().buttons[index] = button;
        self.close_tray(window, cx);
    }

    fn remove_button(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(Tray::Button(tray)) = &self.tray {
            let index = tray.index;
            self.deck.current_mut().buttons.remove(index);
            self.close_tray(window, cx);
        }
    }
}
