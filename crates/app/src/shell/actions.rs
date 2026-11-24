use super::{Shell, now_ms};
use crate::{
    home::CardEditor,
    presence::{LinkState, Status as Connection},
    tray::Tray,
};
use flexolotl_core::{
    activity::{Activity, ActivityKind},
    settings::{Appearance, Motion},
    share,
    status::{Source, Status, Timing},
};
use gpui_kit::{ClipboardItem, Context, Window};
use std::time::Duration;

const NOTICE_TIME: Duration = Duration::from_millis(1600);

impl Shell {
    pub fn activity(&self) -> Option<Activity> {
        if self.uses_music() {
            return self.music_activity(now_ms());
        }
        let status = self.deck.current();
        Some(status.activity(self.started_ms, self.settings.member_line))
    }

    pub fn can_show(&self) -> bool {
        let is_connected = matches!(
            self.connection.link,
            LinkState::Ready | LinkState::Shown | LinkState::Refused(_)
        );
        is_connected && self.is_complete()
    }

    pub fn is_complete(&self) -> bool {
        if self.uses_music() {
            return true;
        }
        let has_title = !self.deck.current().title.trim().is_empty();
        let is_valid = self
            .activity()
            .is_some_and(|activity| activity.issues().is_empty());
        has_title && is_valid
    }

    pub fn toggle_live(&mut self, cx: &mut Context<Self>) {
        if self.is_live {
            self.take_off(cx);
        } else if self.can_show() {
            self.put_on(cx);
        }
    }

    pub fn status_changed(&mut self, cx: &mut Context<Self>) {
        self.save_deck();
        self.publish();
        cx.notify();
    }

    pub fn choose_kind(&mut self, kind: ActivityKind, window: &mut Window, cx: &mut Context<Self>) {
        let status = self.deck.current_mut();
        status.kind = kind;
        status.source = Source::Text;
        self.tune_music(cx);
        self.close_tray(window, cx);
    }

    pub fn choose_music(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.deck.current_mut().source = Source::Music;
        self.tune_music(cx);
        self.close_tray(window, cx);
    }

    pub fn set_timing(&mut self, timing: Timing, cx: &mut Context<Self>) {
        self.deck.current_mut().timing = timing;
        self.started_ms = now_ms();
        self.status_changed(cx);
    }

    pub fn open_status(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.deck.select(index);
        self.current_replaced(window, cx);
        self.go_home(cx);
    }

    pub fn add_status(&mut self, status: Status, window: &mut Window, cx: &mut Context<Self>) {
        let is_blank = status.is_blank();
        if is_blank && self.is_live {
            self.take_off(cx);
        }
        self.deck.add(status);
        self.current_replaced(window, cx);
        self.go_home(cx);
        if is_blank {
            self.editor().title.focus(window, cx);
        }
    }

    pub fn paste_status(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let pasted = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .and_then(|text| share::decode(&text));
        match pasted {
            Some(status) => self.add_status(status, window, cx),
            None => self.show_notice(self.copy().bad_code, cx),
        }
    }

    pub fn copy_status(&mut self, index: usize, cx: &mut Context<Self>) {
        let code = share::encode(&self.deck.statuses()[index]);
        cx.write_to_clipboard(ClipboardItem::new_string(code));
        self.show_notice(self.copy().copied, cx);
    }

    pub fn delete_status(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.deck.remove(index);
        self.carousel.settle_on(self.deck.current_index());
        self.current_replaced(window, cx);
    }

    pub fn close_tray(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(Tray::Button(tray)) = &self.tray
            && tray.is_new
        {
            let index = tray.index;
            self.deck.current_mut().buttons.remove(index);
        }
        self.tray = None;
        self.local_picture = None;
        self.picture_problem = None;
        window.focus(&self.focus.clone(), cx);
        self.status_changed(cx);
    }

    pub fn settings_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.saver.save_settings(&self.settings);
        self.retarget_reveal();
        self.editor = Some(CardEditor::new(
            self.deck.current(),
            self.copy(),
            window,
            cx,
        ));
        self.publish();
        cx.notify();
    }

    pub fn set_application_id(&mut self, id: String, window: &mut Window, cx: &mut Context<Self>) {
        self.settings.application_id = id;
        self.is_live = false;
        self.connection = Connection::default();
        let (presence, listener) = Self::listen(self.settings.application_id(), cx);
        self.presence = presence;
        self._listener = listener;
        self.settings_changed(window, cx);
    }

    pub(super) fn receive(&mut self, connection: Connection, cx: &mut Context<Self>) {
        if matches!(connection.link, LinkState::Refused(_)) {
            self.is_live = false;
            self.retarget_reveal();
        }
        self.connection = connection;
        if std::mem::take(&mut self.waits_for_start) && self.can_show() {
            self.put_on(cx);
        }
        cx.notify();
    }

    pub(super) fn retarget_reveal(&mut self) {
        let is_dark = match self.settings.appearance {
            Appearance::Auto => self.is_live,
            Appearance::Light => false,
            Appearance::Dark => true,
        };
        self.reveal.go_to(f32::from(is_dark));
        if self.settings.motion == Motion::Reduced {
            self.reveal.finish();
        }
    }

    pub(super) fn shows_running_time(&self) -> bool {
        self.uses_music() || self.deck.current().timing != Timing::Off
    }

    fn put_on(&mut self, cx: &mut Context<Self>) {
        self.is_live = true;
        self.started_ms = now_ms();
        self.publish();
        self.retarget_reveal();
        cx.notify();
    }

    fn take_off(&mut self, cx: &mut Context<Self>) {
        self.is_live = false;
        self.presence.hide();
        self.retarget_reveal();
        cx.notify();
    }

    pub fn publish(&self) {
        if !self.is_live || self.uses_music() && self.music.is_settling() {
            return;
        }
        let payload = self
            .activity()
            .filter(|_| self.is_complete())
            .and_then(|activity| activity.payload().ok());
        match payload {
            Some(payload) => self.presence.show(payload),
            None if self.uses_music() => self.presence.hide(),
            None => {}
        }
    }

    fn current_replaced(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.started_ms = now_ms();
        self.editor().load(self.deck.current(), window, cx);
        self.tune_music(cx);
        self.status_changed(cx);
    }

    fn save_deck(&self) {
        self.saver.save_deck(&self.deck);
    }

    fn show_notice(&mut self, text: &'static str, cx: &mut Context<Self>) {
        self.notice = Some(text.into());
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(NOTICE_TIME).await;
            let _ = this.update(cx, |shell, cx| {
                shell.notice = None;
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}
