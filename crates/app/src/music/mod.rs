mod activity;
mod lookup;
mod player;
mod system;
mod track;
mod video;

pub use player::Player;
pub use track::Track;

use crate::shell::Shell;
use flexolotl_core::{lyrics::Lyrics, status::Source};
use gpui_kit::{Context, Task};
use std::time::Duration;

const LINE_CHECK: Duration = Duration::from_millis(400);

pub const HEARS_BROWSERS: bool = cfg!(not(target_os = "macos"));

#[derive(Default)]
pub struct Music {
    pub track: Option<Track>,
    lyrics: Option<(String, Lyrics)>,
    cover: Option<(String, String)>,
    line: Option<String>,
    player: Option<Player>,
    tasks: Vec<Task<()>>,
    lookup: Option<Task<()>>,
}

impl Music {
    pub fn line(&self) -> Option<&str> {
        self.line.as_deref()
    }

    pub fn cover(&self) -> Option<&str> {
        let track = self.track.as_ref()?;
        track.cover.as_deref().or_else(|| {
            let (key, cover) = self.cover.as_ref()?;
            (*key == track.key()).then_some(cover.as_str())
        })
    }

    pub fn is_video(&self) -> bool {
        self.lyrics.is_none()
            && self
                .track
                .as_ref()
                .is_some_and(|track| track.is_from_browser)
    }

    fn current_line(&self) -> Option<String> {
        let track = self.track.as_ref()?;
        let (key, lyrics) = self.lyrics.as_ref()?;
        if *key != track.key() {
            return None;
        }
        lyrics.line_at(track.position_ms()).map(str::to_owned)
    }
}

impl Shell {
    pub fn uses_music(&self) -> bool {
        self.deck.current().source == Source::Music
    }

    pub fn tune_music(&mut self, cx: &mut Context<Self>) {
        if !self.uses_music() {
            self.music = Music::default();
            return;
        }
        if self.music.player.is_none() {
            self.start_music(cx);
        }
    }

    pub fn restart_music(&mut self, cx: &mut Context<Self>) {
        self.music = Music::default();
        self.tune_music(cx);
        self.publish();
    }

    fn start_music(&mut self, cx: &mut Context<Self>) {
        let (player, tracks) = Player::start(self.settings.music_in_browsers);
        let listener = cx.spawn(async move |this, cx| {
            while let Ok(track) = tracks.recv().await {
                let update = this.update(cx, |shell, cx| shell.receive_track(track, cx));
                if update.is_err() {
                    break;
                }
            }
        });
        let clock = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(LINE_CHECK).await;
                let update = this.update(cx, |shell, cx| shell.refresh_line(cx));
                if update.is_err() {
                    break;
                }
            }
        });
        self.music.player = Some(player);
        self.music.tasks = vec![listener, clock];
    }

    fn receive_track(&mut self, track: Option<Track>, cx: &mut Context<Self>) {
        let key = track.as_ref().map(Track::key);
        let is_new = key != self.music.track.as_ref().map(Track::key);
        self.music.track = track;
        if is_new {
            self.music.lyrics = None;
            self.music.cover = None;
            self.music.line = None;
            self.look_up_lyrics(cx);
        }
        self.publish();
        cx.notify();
    }

    fn refresh_line(&mut self, cx: &mut Context<Self>) {
        let line = self.music.current_line();
        if line != self.music.line {
            self.music.line = line;
            self.publish();
            cx.notify();
        }
    }

    fn look_up_lyrics(&mut self, cx: &mut Context<Self>) {
        self.music.lookup = None;
        let Some(track) = self.music.track.clone() else {
            return;
        };
        if !self.settings.music_online {
            return;
        }
        let client = cx.http_client();
        self.music.lookup = Some(cx.spawn(async move |this, cx| {
            let found = lookup::find(client, &track).await;
            let _ = this.update(cx, |shell, cx| {
                let key = track.key();
                shell.music.lyrics = found.lyrics.map(|lyrics| (key.clone(), lyrics));
                shell.music.cover = found.cover.map(|cover| (key, cover));
                shell.refresh_line(cx);
                shell.publish();
                cx.notify();
            });
        }));
    }
}
