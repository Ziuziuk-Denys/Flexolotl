use crate::{Plugin, host::Fetch, script::Script};
use flexolotl_core::activity::Activity;
use std::{
    sync::mpsc::{self, RecvTimeoutError},
    thread,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Shows(Option<Box<Activity>>),
    Failed(String),
}

pub struct Runner {
    _running: mpsc::Sender<()>,
}

impl Runner {
    pub fn start(plugin: Plugin, fetch: Fetch) -> (Self, async_channel::Receiver<Event>) {
        let (running, stops) = mpsc::channel();
        let (outbox, events) = async_channel::unbounded();
        thread::spawn(move || run(&plugin, fetch, &stops, &outbox));
        (Self { _running: running }, events)
    }
}

fn run(
    plugin: &Plugin,
    fetch: Fetch,
    stops: &mpsc::Receiver<()>,
    outbox: &async_channel::Sender<Event>,
) {
    let script = match Script::start(plugin, fetch) {
        Ok(script) => script,
        Err(problem) => {
            let _ = outbox.send_blocking(Event::Failed(problem));
            return;
        }
    };
    let mut shown = None;
    loop {
        let event = match script.update() {
            Ok(activity) => Event::Shows(activity.map(Box::new)),
            Err(problem) => Event::Failed(problem),
        };
        if shown.as_ref() != Some(&event) {
            shown = Some(event.clone());
            if outbox.send_blocking(event).is_err() {
                return;
            }
        }
        match stops.recv_timeout(script.pause()) {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Manifest;
    use std::sync::Arc;

    #[test]
    fn tells_only_about_changes_and_stops_when_dropped() {
        let manifest = r#"{"id":"test","name":"Test","version":"1","api":1}"#;
        let plugin = Plugin {
            manifest: Manifest::parse(manifest).unwrap(),
            script: "local n = 0 return { every = 1, update = function() n = n + 1 if n == 1 then return { title = 'one' } end error('broken') end }".into(),
        };
        let (runner, events) = Runner::start(plugin, Arc::new(|_| None));
        let Ok(Event::Shows(Some(first))) = events.recv_blocking() else {
            panic!("the first update shows a status");
        };
        assert_eq!(first.name, "one");
        assert!(matches!(events.recv_blocking(), Ok(Event::Failed(_))));
        drop(runner);
        assert!(events.recv_blocking().is_err());
    }
}
