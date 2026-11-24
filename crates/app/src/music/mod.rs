mod activity;
mod finds;
mod lookup;
mod player;
mod system;
mod track;
mod video;

pub use player::Player;
pub use track::Track;

use crate::shell::Shell;
use finds::{Find, Finds, Search};
use flexolotl_core::status::Source;
use gpui_kit::{Context, Task};
use std::time::Duration;

const LINE_CHECK: Duration = Duration::from_millis(100);
const SETTLING: Duration = Duration::from_millis(700);

pub const HEARS_BROWSERS: bool = cfg!(not(target_os = "macos"));

#[derive(Default)]
pub struct Music {
    pub track: Option<Track>,
    line: Option<String>,
    player: Option<Player>,
    tasks: Vec<Task<()>>,
    lookups: Vec<Task<()>>,
    settling: Option<Task<()>>,
    finds: Finds,
}

impl Music {
    pub fn line(&self) -> Option<&str> {
        self.line.as_deref()
    }

    pub fn cover(&self) -> Option<&str> {
        let own = self.track.as_ref()?.cover.as_deref();
        own.or_else(|| self.find()?.cover.found().map(String::as_str))
    }

    pub fn has_no_lyrics(&self) -> bool {
        let is_missing = self
            .find()
            .is_some_and(|find| matches!(find.lyrics, Search::Missing));
        is_missing && !self.is_video()
    }

    pub fn is_video(&self) -> bool {
        let is_from_browser = self
            .track
            .as_ref()
            .is_some_and(|track| track.is_from_browser);
        let has_lyrics = self
            .find()
            .is_some_and(|find| find.lyrics.found().is_some());
        is_from_browser && !has_lyrics
    }

    pub fn is_settling(&self) -> bool {
        self.settling.is_some()
    }

    fn find(&self) -> Option<&Find> {
        self.finds.get(&self.track.as_ref()?.key())
    }

    fn current_line(&self) -> Option<String> {
        let track = self.track.as_ref()?;
        let lyrics = self.find()?.lyrics.found()?;
        lyrics.line_at(track.position_ms()).map(str::to_owned)
    }

    fn stop(&mut self) {
        let mut finds = std::mem::take(&mut self.finds);
        finds.drop_unfinished();
        *self = Self {
            finds,
            ..Self::default()
        };
    }
}

impl Shell {
    pub fn uses_music(&self) -> bool {
        self.deck.current().source == Source::Music
    }

    pub fn tune_music(&mut self, cx: &mut Context<Self>) {
        if !self.uses_music() {
            self.music.stop();
            return;
        }
        if self.music.player.is_none() {
            self.start_music(cx);
        }
    }

    pub fn restart_music(&mut self, cx: &mut Context<Self>) {
        self.music.stop();
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
            self.music.settling = None;
            self.look_up_track(cx);
            self.music.line = self.music.current_line();
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

    fn look_up_track(&mut self, cx: &mut Context<Self>) {
        self.music.lookups.clear();
        self.music.finds.drop_unfinished();
        let Some(track) = self.music.track.clone() else {
            return;
        };
        let key = track.key();
        if !self.settings.music_online || self.music.finds.get(&key).is_some() {
            return;
        }
        let needs_cover = track.cover.is_none();
        self.music.finds.start(key.clone(), needs_cover);
        let (client, song, name) = (cx.http_client(), track.clone(), key.clone());
        let lyrics = cx.spawn(async move |this, cx| {
            let lyrics = lookup::find_lyrics(client, song).await;
            let _ = this.update(cx, |shell, cx| {
                if let Some(find) = shell.music.finds.get_mut(&name) {
                    find.lyrics = lyrics.into();
                }
                shell.track_found(&name, cx);
            });
        });
        let client = cx.http_client();
        let cover = cx.spawn(async move |this, cx| {
            if !needs_cover {
                return;
            }
            let cover = lookup::find_cover(client, track).await;
            let _ = this.update(cx, |shell, cx| {
                if let Some(find) = shell.music.finds.get_mut(&key) {
                    find.cover = cover.into();
                }
                shell.track_found(&key, cx);
            });
        });
        self.music.lookups = vec![lyrics, cover];
        self.music.settling = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SETTLING).await;
            let _ = this.update(cx, |shell, _| {
                shell.music.settling = None;
                shell.publish();
            });
        }));
    }

    fn track_found(&mut self, key: &str, cx: &mut Context<Self>) {
        let find = self.music.finds.get(key);
        if find.is_some_and(Find::is_complete) {
            self.music.settling = None;
        }
        self.music.line = self.music.current_line();
        self.publish();
        cx.notify();
    }
}
