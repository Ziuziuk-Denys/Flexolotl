mod rows;

use crate::{platform::autostart, presence::LinkState, shell::Shell, theme::Theme};
use flexolotl_core::{
    activity::MemberLine,
    settings::{Appearance, Language, Lines, Motion},
};
use gpui_kit::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px,
};
use rows::{choice, heading, link, note, switch};

const MEMBER_LINES: [MemberLine; 2] = [MemberLine::Name, MemberLine::Details];

impl Shell {
    pub fn render_settings(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::mixed(self.reveal().round());
        div()
            .size_full()
            .pt(px(60.))
            .text_color(theme.ink)
            .child(
                div()
                    .id("settings")
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
                            .child(self.copy().settings),
                    )
                    .children(self.general_rows(theme, cx))
                    .children(self.music_rows(theme, cx))
                    .children(self.look_rows(theme, cx))
                    .children(self.discord_rows(theme, cx))
                    .children(self.about_rows(theme)),
            )
            .into_any_element()
    }

    fn general_rows(&self, theme: Theme, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let copy = self.copy();
        let language = self.settings.language;
        vec![
            heading(copy.general, theme),
            switch(
                "launch",
                copy.launch_at_login,
                autostart::is_enabled(),
                theme,
            )
            .on_click(cx.listener(|_, _, _, cx| {
                let _ = autostart::set_enabled(!autostart::is_enabled());
                cx.notify();
            }))
            .into_any_element(),
            switch(
                "show-on-start",
                copy.show_on_start,
                self.settings.show_on_start,
                theme,
            )
            .on_click(cx.listener(|shell, _, window, cx| {
                shell.settings.show_on_start ^= true;
                shell.settings_changed(window, cx);
            }))
            .into_any_element(),
            switch(
                "background",
                copy.stays_in_background,
                self.settings.stays_in_background,
                theme,
            )
            .on_click(cx.listener(|shell, _, window, cx| {
                shell.settings.stays_in_background ^= true;
                shell.settings_changed(window, cx);
            }))
            .into_any_element(),
            note(copy.background_note, theme),
            choice(
                "language",
                copy.language,
                copy.languages[language as usize],
                theme,
            )
            .on_click(cx.listener(move |shell, _, window, cx| {
                shell.settings.language = following(Language::ALL, language);
                shell.settings_changed(window, cx);
            }))
            .into_any_element(),
        ]
    }

    fn music_rows(&self, theme: Theme, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let copy = self.copy();
        vec![
            heading(copy.music, theme),
            switch(
                "lyrics",
                copy.shows_lyrics,
                self.settings.music_online,
                theme,
            )
            .on_click(cx.listener(|shell, _, window, cx| {
                shell.settings.music_online ^= true;
                shell.restart_music(cx);
                shell.settings_changed(window, cx);
            }))
            .into_any_element(),
            note(copy.lyrics_note, theme),
            switch(
                "browsers",
                copy.hears_browsers,
                self.settings.music_in_browsers,
                theme,
            )
            .on_click(cx.listener(|shell, _, window, cx| {
                shell.settings.music_in_browsers ^= true;
                shell.restart_music(cx);
                shell.settings_changed(window, cx);
            }))
            .into_any_element(),
            note(copy.browsers_note, theme),
        ]
    }

    fn look_rows(&self, theme: Theme, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let copy = self.copy();
        let settings = &self.settings;
        let (appearance, lines, motion) = (settings.appearance, settings.lines, settings.motion);
        vec![
            heading(copy.look, theme),
            choice(
                "appearance",
                copy.appearance,
                copy.appearances[appearance as usize],
                theme,
            )
            .on_click(cx.listener(move |shell, _, window, cx| {
                shell.settings.appearance = following(Appearance::ALL, appearance);
                shell.settings_changed(window, cx);
            }))
            .into_any_element(),
            note(copy.appearance_note, theme),
            choice(
                "lines",
                copy.lines,
                copy.line_options[lines as usize],
                theme,
            )
            .on_click(cx.listener(move |shell, _, window, cx| {
                shell.settings.lines = following(Lines::ALL, lines);
                shell.settings_changed(window, cx);
            }))
            .into_any_element(),
            choice(
                "motion",
                copy.motion,
                copy.motion_options[motion as usize],
                theme,
            )
            .on_click(cx.listener(move |shell, _, window, cx| {
                shell.settings.motion = following(Motion::ALL, motion);
                shell.settings_changed(window, cx);
            }))
            .into_any_element(),
        ]
    }

    fn discord_rows(&self, theme: Theme, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let copy = self.copy();
        let line = self.settings.member_line;
        let line_index = usize::from(line == MemberLine::Details);
        let application = match self.settings.application_id.trim() {
            "" => "Flexolotl".to_owned(),
            id => id.to_owned(),
        };
        let mut rows = vec![
            heading(copy.profile, theme),
            choice(
                "member-line",
                copy.member_line,
                copy.member_lines[line_index],
                theme,
            )
            .on_click(cx.listener(move |shell, _, window, cx| {
                shell.settings.member_line = following(&MEMBER_LINES, line);
                shell.settings_changed(window, cx);
            }))
            .into_any_element(),
            note(copy.member_line_note, theme),
            choice("application", copy.application, application, theme)
                .on_click(cx.listener(|shell, _, window, cx| {
                    shell.open_application_tray(window, cx);
                }))
                .into_any_element(),
            note(copy.application_note, theme),
            link("doctor", copy.doctor, copy.doctor_check, theme)
                .on_click(cx.listener(|shell, _, _, cx| {
                    shell.is_checking = true;
                    cx.notify();
                }))
                .into_any_element(),
        ];
        if self.is_checking {
            rows.extend(self.findings().map(|finding| note(finding, theme)));
        }
        rows
    }

    fn findings(&self) -> [&'static str; 3] {
        let copy = self.copy();
        let is_running = !matches!(
            self.connection.link,
            LinkState::DiscordClosed | LinkState::Connecting
        );
        let is_shown = self.connection.link == LinkState::Shown && self.is_live;
        [
            copy.doctor_running[usize::from(is_running)],
            copy.doctor_shown[usize::from(is_shown)],
            copy.doctor_switch,
        ]
    }

    fn about_rows(&self, theme: Theme) -> Vec<AnyElement> {
        let copy = self.copy();
        vec![
            heading(copy.about, theme),
            rows::fact(copy.version, env!("CARGO_PKG_VERSION"), theme),
        ]
    }
}

fn following<T: Copy + PartialEq>(options: &[T], current: T) -> T {
    let index = options
        .iter()
        .position(|option| *option == current)
        .map_or(0, |index| (index + 1) % options.len());
    options[index]
}

#[cfg(test)]
mod tests {
    use super::following;

    #[test]
    fn goes_around_the_options() {
        assert_eq!(following(&[1, 2, 3], 1), 2);
        assert_eq!(following(&[1, 2, 3], 3), 1);
        assert_eq!(following(&[1, 2, 3], 9), 1);
    }
}
