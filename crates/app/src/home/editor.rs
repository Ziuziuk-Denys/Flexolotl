use super::palette;
use crate::{
    copy::Copy,
    shell::Shell,
    widgets::{FieldColors, FieldEvent, TextField},
};
use flexolotl_core::status::Status;
use gpui_kit::{Context, Window};

pub struct CardEditor {
    pub title: TextField,
    pub subtitle: TextField,
}

impl CardEditor {
    pub fn new(
        status: &Status,
        copy: &'static Copy,
        window: &mut Window,
        cx: &mut Context<Shell>,
    ) -> Self {
        let colors = FieldColors {
            ink: palette::text().into(),
            soft: palette::hint().into(),
            selection: palette::selection().into(),
        };
        Self {
            title: TextField::new(
                copy.title_hint,
                status.title.clone(),
                colors,
                Shell::edit_card,
                window,
                cx,
            ),
            subtitle: TextField::new(
                copy.subtitle_hint,
                status.subtitle.clone(),
                colors,
                Shell::edit_card,
                window,
                cx,
            ),
        }
    }

    pub fn load(&self, status: &Status, window: &mut Window, cx: &mut Context<Shell>) {
        self.title.set_text(&status.title, window, cx);
        self.subtitle.set_text(&status.subtitle, window, cx);
    }
}

impl Shell {
    fn edit_card(&mut self, event: FieldEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            FieldEvent::Submitted => window.blur(cx),
            FieldEvent::Changed => {
                let title = self.editor().title.text(cx);
                let subtitle = self.editor().subtitle.text(cx);
                let status = self.deck.current_mut();
                status.title = title;
                status.subtitle = subtitle;
                self.status_changed(cx);
            }
            FieldEvent::Left => cx.notify(),
        }
    }
}
