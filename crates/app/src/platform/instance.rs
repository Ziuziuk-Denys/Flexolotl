use super::paths;
use crate::background::Request;
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
const TOKEN_LENGTH: usize = 32;
const ACCEPTED: &[u8; 2] = b"ok";

#[derive(Debug, PartialEq, Eq)]
pub enum Instance {
    First,
    Second,
}

pub fn claim(requests: async_channel::Sender<Request>) -> Instance {
    if hand_over().is_some() {
        return Instance::Second;
    }
    if let Some((listener, token)) = announce() {
        thread::spawn(move || listen(&listener, &token, &requests));
    }
    Instance::First
}

fn hand_over() -> Option<()> {
    let note = fs::read_to_string(paths::instance_file()).ok()?;
    let (port, token) = note.split_once(' ')?;
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port.parse().ok()?));
    let mut stream = TcpStream::connect_timeout(&address, ANSWER_TIMEOUT).ok()?;
    stream.set_read_timeout(Some(ANSWER_TIMEOUT)).ok()?;
    stream.write_all(token.as_bytes()).ok()?;
    let mut answer = [0; ACCEPTED.len()];
    stream.read_exact(&mut answer).ok()?;
    (&answer == ACCEPTED).then_some(())
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
        let mut received = [0; TOKEN_LENGTH];
        let is_known = stream.set_read_timeout(Some(ANSWER_TIMEOUT)).is_ok()
            && stream.read_exact(&mut received).is_ok()
            && received == token.as_bytes();
        if is_known {
            let _ = stream.write_all(ACCEPTED);
            if requests.send_blocking(Request::Open).is_err() {
                return;
            }
        }
    }
}

fn random_token() -> String {
    let part = || RandomState::new().build_hasher().finish();
    format!("{:016x}{:016x}", part(), part())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_a_fresh_token_every_time() {
        let (first, second) = (random_token(), random_token());
        assert_eq!(first.len(), TOKEN_LENGTH);
        assert_ne!(first, second);
    }
}
