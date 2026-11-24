use super::{Tray, done_button, field_colors, labelled};
use crate::{
    copy::Copy,
    shell::Shell,
    widgets::{FieldEvent, TextField},
};
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement, StatefulInteractiveElement, Styled, Window,
    div,
};

pub fn render(copy: &'static Copy, field: &TextField, cx: &mut Context<Shell>) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .child(labelled(copy.application_label, field))
        .child(
            done_button(copy.done)
                .on_click(cx.listener(|shell, _, window, cx| shell.finish_application(window, cx))),
        )
        .into_any_element()
}

impl Shell {
    pub fn open_application_tray(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let field = TextField::new(
            self.copy().application_hint,
            self.settings.application_id.clone(),
            field_colors(),
            Self::edit_application,
            window,
            cx,
        );
        field.focus(window, cx);
        self.open_tray(Tray::Application(field), cx);
    }

    fn edit_application(&mut self, event: FieldEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event == FieldEvent::Submitted {
            self.finish_application(window, cx);
        }
    }

    fn finish_application(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Tray::Application(field)) = &self.tray else {
            return;
        };
        let id = field.text(cx).trim().to_owned();
        self.close_tray(window, cx);
        if id != self.settings.application_id {
            self.set_application_id(id, window, cx);
        }
    }
}
