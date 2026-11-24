use flexolotl_core::{settings::Settings, status::Deck, store::Store};
use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
};

enum Job {
    Deck(Deck),
    Settings(Settings),
}

pub struct Saver {
    jobs: Option<mpsc::Sender<Job>>,
    thread: Option<JoinHandle<()>>,
}

impl Saver {
    pub fn start(store: Store) -> Self {
        let (jobs, inbox) = mpsc::channel::<Job>();
        let thread = thread::spawn(move || {
            while let Ok(first) = inbox.recv() {
                let (mut deck, mut settings) = (None, None);
                for job in std::iter::once(first).chain(inbox.try_iter()) {
                    match job {
                        Job::Deck(value) => deck = Some(value),
                        Job::Settings(value) => settings = Some(value),
                    }
                }
                if let Some(deck) = deck {
                    let _ = store.save("deck", &deck);
                }
                if let Some(settings) = settings {
                    let _ = store.save("settings", &settings);
                }
            }
        });
        Self {
            jobs: Some(jobs),
            thread: Some(thread),
        }
    }

    pub fn save_deck(&self, deck: &Deck) {
        self.send(Job::Deck(deck.clone()));
    }

    pub fn save_settings(&self, settings: &Settings) {
        self.send(Job::Settings(settings.clone()));
    }

    pub fn finish(&mut self) {
        drop(self.jobs.take());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }

    fn send(&self, job: Job) {
        if let Some(jobs) = &self.jobs {
            let _ = jobs.send(job);
        }
    }
}

impl Drop for Saver {
    fn drop(&mut self) {
        self.finish();
    }
}
