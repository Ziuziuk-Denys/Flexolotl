use super::{LinkState, Status};
use flexolotl_core::rpc::{Connection, RpcError, Throttle};
use serde_json::Value;
use std::{
    sync::mpsc::{self, RecvTimeoutError},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const HEARTBEAT: Duration = Duration::from_secs(10);

enum Command {
    Show(Value),
    Hide,
}

pub struct Presence {
    commands: Option<mpsc::Sender<Command>>,
    thread: Option<JoinHandle<()>>,
}

impl Presence {
    pub fn start(application_id: &str) -> (Self, async_channel::Receiver<Status>) {
        let (commands, inbox) = mpsc::channel();
        let (outbox, statuses) = async_channel::unbounded();
        let application_id = application_id.to_owned();
        let thread = thread::spawn(move || Worker::new(application_id, inbox, outbox).run());
        let presence = Self {
            commands: Some(commands),
            thread: Some(thread),
        };
        (presence, statuses)
    }

    pub fn show(&self, payload: Value) {
        self.send(Command::Show(payload));
    }

    pub fn hide(&self) {
        self.send(Command::Hide);
    }

    pub fn finish(&mut self) {
        drop(self.commands.take());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }

    fn send(&self, command: Command) {
        if let Some(commands) = &self.commands {
            let _ = commands.send(command);
        }
    }
}

impl Drop for Presence {
    fn drop(&mut self) {
        self.finish();
    }
}

struct Worker {
    application_id: String,
    inbox: mpsc::Receiver<Command>,
    outbox: async_channel::Sender<Status>,
    connection: Option<Connection>,
    shown: Option<Value>,
    updates: Throttle<Value>,
    status: Status,
}

impl Worker {
    fn new(
        application_id: String,
        inbox: mpsc::Receiver<Command>,
        outbox: async_channel::Sender<Status>,
    ) -> Self {
        Self {
            application_id,
            inbox,
            outbox,
            connection: None,
            shown: None,
            updates: Throttle::default(),
            status: Status::default(),
        }
    }

    fn run(mut self) {
        loop {
            self.step();
            let wait = self.updates.wait(Instant::now()).unwrap_or(HEARTBEAT);
            let wait = if self.connection.is_some() {
                wait
            } else {
                HEARTBEAT
            };
            match self.inbox.recv_timeout(wait) {
                Ok(Command::Show(payload)) => self.show(payload),
                Ok(Command::Hide) => self.hide(),
                Err(RecvTimeoutError::Timeout) => self.check(),
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        self.hide();
    }

    fn step(&mut self) {
        if self.connection.is_none() {
            self.connect();
        }
        let Some(connection) = &mut self.connection else {
            return;
        };
        let Some(payload) = self.updates.take_due(Instant::now()) else {
            return;
        };
        match connection.set_activity(&payload) {
            Ok(_) => self.report(LinkState::Shown),
            Err(error) => self.fail(error),
        }
    }

    fn connect(&mut self) {
        match Connection::open(&self.application_id) {
            Ok(connection) => {
                self.status.account = connection.account().cloned();
                self.connection = Some(connection);
                if let Some(payload) = self.shown.clone() {
                    self.updates.push(payload);
                }
                self.report(LinkState::Ready);
            }
            Err(error) => self.fail(error),
        }
    }

    fn check(&mut self) {
        if self.updates.wait(Instant::now()).is_some() {
            return;
        }
        if let Some(connection) = &mut self.connection
            && let Err(error) = connection.check()
        {
            self.fail(error);
        }
    }

    fn show(&mut self, payload: Value) {
        self.shown = Some(payload.clone());
        self.updates.push(payload);
    }

    fn hide(&mut self) {
        self.updates.clear();
        if self.shown.take().is_none() {
            return;
        }
        if let Some(connection) = &mut self.connection {
            match connection.clear_activity() {
                Ok(()) => self.report(LinkState::Ready),
                Err(error) => self.fail(error),
            }
        }
    }

    fn fail(&mut self, error: RpcError) {
        let link = match error {
            RpcError::Rejected { message, .. } => {
                self.shown = None;
                LinkState::Refused(message)
            }
            _ => {
                self.connection = None;
                LinkState::DiscordClosed
            }
        };
        self.report(link);
    }

    fn report(&mut self, link: LinkState) {
        self.status.link = link;
        let _ = self.outbox.try_send(self.status.clone());
    }
}
