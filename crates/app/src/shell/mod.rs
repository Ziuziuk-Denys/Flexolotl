mod actions;
mod frame;
mod header;
mod navigation;
mod portrait;

pub use navigation::Screen;

use crate::{
    art::Portrait,
    copy::{self, Copy},
    home::CardEditor,
    motion::{FrameClock, Spring, Tween},
    music::Music,
    pictures::{LocalPicture, PictureProblem},
    platform::paths,
    presence::{Presence, Status as Connection},
    saver::Saver,
    statuses::Carousel,
    tray::Tray,
};
use flexolotl_core::{
    settings::{Appearance, Settings},
    status::Deck,
    store::Store,
};
use gpui_kit::{Context, FocusHandle, SharedString, Task, Window};
use std::time::Duration;

const REVEAL_SECONDS: f32 = 0.85;
const CLOCK_TICK: Duration = Duration::from_secs(1);

pub struct Shell {
    pub deck: Deck,
    pub settings: Settings,
    pub connection: Connection,
    pub is_live: bool,
    pub started_ms: u64,
    pub music: Music,
    pub local_picture: Option<LocalPicture>,
    pub picture_problem: Option<PictureProblem>,
    editor: Option<CardEditor>,
    saver: Saver,
    pub tray: Option<Tray>,
    pub carousel: Carousel,
    pub portrait: Portrait,
    pub notice: Option<SharedString>,
    pub is_checking: bool,
    presence: Presence,
    screen: Screen,
    previous_screen: Screen,
    page: Spring,
    reveal: Tween,
    clock: FrameClock,
    focus: FocusHandle,
    waits_for_start: bool,
    _listener: Task<()>,
}

impl Shell {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let store = Store::new(paths::config_directory());
        let settings: Settings = store.load("settings").unwrap_or_default();
        let copy = copy::for_language(settings.language);
        let deck: Deck = store
            .load("deck")
            .unwrap_or_else(|| Deck::starting_with(copy.first_title));
        let (presence, listener) = Self::listen(settings.application_id(), cx);
        Self::start_clock(cx);
        let mut shell = Self {
            editor: None,
            music: Music::default(),
            local_picture: None,
            picture_problem: None,
            saver: Saver::start(store),
            carousel: Carousel::new(deck.current_index()),
            reveal: Tween::resting_at(
                f32::from(settings.appearance == Appearance::Dark),
                REVEAL_SECONDS,
            ),
            waits_for_start: settings.show_on_start,
            deck,
            settings,
            connection: Connection::default(),
            is_live: false,
            started_ms: now_ms(),
            tray: None,
            portrait: Portrait::placeholder(),
            notice: None,
            is_checking: false,
            presence,
            screen: Screen::Home,
            previous_screen: Screen::Home,
            page: Spring::resting_at(1., 0.002),
            clock: FrameClock::default(),
            focus: cx.focus_handle(),
            _listener: listener,
        };
        shell.tune_music(cx);
        shell
    }

    pub fn attach(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.tray = None;
        self.editor = Some(CardEditor::new(
            self.deck.current(),
            self.copy(),
            window,
            cx,
        ));
        self.settle();
    }

    pub fn editor(&self) -> &CardEditor {
        self.editor.as_ref().expect("the shell has a window")
    }

    pub fn finish(&mut self) {
        self.presence.finish();
        self.saver.finish();
    }

    pub fn copy(&self) -> &'static Copy {
        copy::for_language(self.settings.language)
    }

    fn listen(application_id: &str, cx: &mut Context<Self>) -> (Presence, Task<()>) {
        let (presence, statuses) = Presence::start(application_id);
        let listener = cx.spawn(async move |this, cx| {
            while let Ok(status) = statuses.recv().await {
                let update = this.update(cx, |shell, cx| shell.receive(status, cx));
                if update.is_err() {
                    break;
                }
            }
        });
        (presence, listener)
    }

    fn start_clock(cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(CLOCK_TICK).await;
                let update = this.update(cx, |shell, cx| {
                    if shell.shows_running_time() {
                        cx.notify();
                    }
                });
                if update.is_err() {
                    break;
                }
            }
        })
        .detach();
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}
