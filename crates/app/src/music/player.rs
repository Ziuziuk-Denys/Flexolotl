use super::{Track, system};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

pub struct Player {
    stops: Arc<AtomicBool>,
}

impl Player {
    pub fn start(hears_browsers: bool) -> (Self, async_channel::Receiver<Option<Track>>) {
        let (outbox, tracks) = async_channel::unbounded();
        let stops = Arc::new(AtomicBool::new(false));
        let watched = stops.clone();
        thread::spawn(move || watch(hears_browsers, &watched, &outbox));
        (Self { stops }, tracks)
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stops.store(true, Ordering::Relaxed);
    }
}

fn watch(hears_browsers: bool, stops: &AtomicBool, outbox: &async_channel::Sender<Option<Track>>) {
    let Some(system) = system::Media::connect() else {
        return;
    };
    let mut shown: Option<Track> = None;
    while !stops.load(Ordering::Relaxed) {
        let found = system.playing(hears_browsers);
        let is_unchanged = match (&found, &shown) {
            (Some(found), Some(shown)) => found.continues(shown),
            (None, None) => true,
            _ => false,
        };
        if !is_unchanged {
            shown.clone_from(&found);
            if outbox.send_blocking(found).is_err() {
                return;
            }
        }
        system.wait();
    }
}
