use super::paths;
use crate::{arguments::Arguments, background::Request};
use std::{
    collections::hash_map::RandomState,
    fs,
    hash::{BuildHasher, Hasher},
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

const ANSWER_TIMEOUT: Duration = Duration::from_secs(1);
const WAIT_TICK: Duration = Duration::from_millis(100);
const LONGEST_WAIT_TICKS: u32 = 100;
const TOKEN_LENGTH: usize = 32;
const ACCEPTED: &[u8; 2] = b"ok";

pub enum Claim {
    Accepted(Option<Note>),
    HandedOver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    address: SocketAddr,
    token: String,
}

impl Note {
    pub fn send(&self, request: Request) -> Option<()> {
        let mut stream = TcpStream::connect_timeout(&self.address, ANSWER_TIMEOUT).ok()?;
        stream.set_read_timeout(Some(ANSWER_TIMEOUT)).ok()?;
        stream.write_all(self.token.as_bytes()).ok()?;
        stream.write_all(&[request as u8]).ok()?;
        let mut answer = [0; ACCEPTED.len()];
        stream.read_exact(&mut answer).ok()?;
        (&answer == ACCEPTED).then_some(())
    }

    fn read() -> Option<Self> {
        Self::parse(&fs::read_to_string(paths::instance_file()).ok()?)
    }

    fn parse(text: &str) -> Option<Self> {
        let (port, token) = text.split_once(' ')?;
        Some(Self {
            address: SocketAddr::from((Ipv4Addr::LOCALHOST, port.parse().ok()?)),
            token: token.to_owned(),
        })
    }

    fn wait_until_gone(&self) {
        for _ in 0..LONGEST_WAIT_TICKS {
            if TcpStream::connect(self.address).is_err() {
                return;
            }
            thread::sleep(WAIT_TICK);
        }
    }
}

pub fn claim(arguments: Arguments, requests: async_channel::Sender<Request>) -> Claim {
    let earlier = Note::read();
    if let Some(earlier) = &earlier {
        if arguments.is_after_update {
            earlier.wait_until_gone();
        } else if !arguments.is_heir && earlier.send(Request::Open).is_some() {
            return Claim::HandedOver;
        }
    }
    if let Some((listener, token)) = announce() {
        thread::spawn(move || listen(&listener, &token, &requests));
    }
    Claim::Accepted(earlier.filter(|_| arguments.is_heir))
}

fn announce() -> Option<(TcpListener, String)> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).ok()?;
    let port = listener.local_addr().ok()?.port();
    let token = random_token();
    fs::create_dir_all(paths::config_directory()).ok()?;
    fs::write(paths::instance_file(), format!("{port} {token}")).ok()?;
    Some((listener, token))
}

fn listen(listener: &TcpListener, token: &str, requests: &async_channel::Sender<Request>) {
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else {
            continue;
        };
        let Some(request) = read_request(&mut stream, token) else {
            continue;
        };
        let _ = stream.write_all(ACCEPTED);
        if requests.send_blocking(request).is_err() {
            return;
        }
    }
}

fn read_request(stream: &mut TcpStream, token: &str) -> Option<Request> {
    let mut received = [0; TOKEN_LENGTH + 1];
    stream.set_read_timeout(Some(ANSWER_TIMEOUT)).ok()?;
    stream.read_exact(&mut received).ok()?;
    let (key, request) = received.split_at(TOKEN_LENGTH);
    if key != token.as_bytes() {
        return None;
    }
    [Request::Open, Request::Quit]
        .into_iter()
        .find(|known| *known as u8 == request[0])
}

fn random_token() -> String {
    let part = || RandomState::new().build_hasher().finish();
    format!("{:016x}{:016x}", part(), part())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_only_to_the_token() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let token = random_token();
        let (sender, requests) = async_channel::unbounded();
        let kept = token.clone();
        thread::spawn(move || listen(&listener, &kept, &sender));
        let note = |token: String| Note { address, token };
        assert_eq!(note(random_token()).send(Request::Quit), None);
        assert_eq!(note(token).send(Request::Quit), Some(()));
        assert_eq!(requests.recv_blocking(), Ok(Request::Quit));
    }

    #[test]
    fn makes_a_fresh_token_every_time() {
        let (first, second) = (random_token(), random_token());
        assert_eq!(first.len(), TOKEN_LENGTH);
        assert_ne!(first, second);
    }
}
