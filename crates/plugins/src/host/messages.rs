use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

const MOST: usize = 100;
const LARGEST: usize = 64 * 1024;
const MOST_HEADERS: usize = 64;
const PATIENCE: Duration = Duration::from_secs(2);
const MARK: &str = "x-flexolotl:";

pub struct Messages {
    queue: Arc<Mutex<VecDeque<String>>>,
    stops: Arc<AtomicBool>,
    address: SocketAddr,
}

impl Messages {
    pub fn listen(port: u16) -> Option<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port)).ok()?;
        let messages = Self {
            queue: Arc::default(),
            stops: Arc::default(),
            address: listener.local_addr().ok()?,
        };
        let (queue, stops) = (messages.queue.clone(), messages.stops.clone());
        thread::spawn(move || {
            for stream in listener.incoming() {
                if stops.load(Ordering::Relaxed) {
                    return;
                }
                if let Ok(stream) = stream {
                    receive(stream, &queue);
                }
            }
        });
        Some(messages)
    }

    pub fn take(&self) -> Vec<String> {
        self.queue
            .lock()
            .map(|mut queue| queue.drain(..).collect())
            .unwrap_or_default()
    }
}

impl Drop for Messages {
    fn drop(&mut self) {
        self.stops.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(self.address);
    }
}

fn receive(mut stream: TcpStream, queue: &Mutex<VecDeque<String>>) {
    let _ = stream.set_read_timeout(Some(PATIENCE));
    let _ = stream.set_write_timeout(Some(PATIENCE));
    let answer = match read_message(&stream) {
        Some(message) => {
            if let Ok(mut queue) = queue.lock() {
                if queue.len() == MOST {
                    queue.pop_front();
                }
                queue.push_back(message);
            }
            "204 No Content"
        }
        None => "400 Bad Request",
    };
    let _ = write!(stream, "HTTP/1.1 {answer}\r\nConnection: close\r\n\r\n");
}

fn read_message(stream: &TcpStream) -> Option<String> {
    let mut reader = BufReader::new(stream.take(2 * LARGEST as u64));
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    if !line.starts_with("POST ") {
        return None;
    }
    let (mut length, mut is_marked) = (0, false);
    for _ in 0..MOST_HEADERS {
        line.clear();
        reader.read_line(&mut line).ok()?;
        let header = line.trim().to_ascii_lowercase();
        if header.is_empty() {
            break;
        }
        is_marked |= header.starts_with(MARK);
        if let Some(value) = header.strip_prefix("content-length:") {
            length = value.trim().parse().ok()?;
        }
    }
    if length > LARGEST {
        return None;
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;
    is_marked.then(|| String::from_utf8(body).ok())?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(address: SocketAddr, headers: &str, body: &str) -> String {
        let mut stream = TcpStream::connect(address).unwrap();
        write!(
            stream,
            "POST / HTTP/1.1\r\n{headers}Content-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        answer
    }

    #[test]
    fn takes_messages_from_games_but_not_from_web_pages() {
        let messages = Messages::listen(0).unwrap();
        assert!(post(messages.address, "X-Flexolotl: 1\r\n", "hello").contains("204"));
        assert!(post(messages.address, "Origin: https://evil.example\r\n", "fake").contains("400"));
        assert_eq!(messages.take(), ["hello"]);
        assert!(messages.take().is_empty());
    }
}
