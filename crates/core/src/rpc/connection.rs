use super::{Account, Frame, Opcode, RpcError, candidates, transport::Stream};
use serde_json::{Value, json};
use std::{path::Path, process, time::Duration};

const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct Connection {
    stream: Stream,
    next_nonce: u64,
    account: Option<Account>,
}

impl Connection {
    pub fn open(application_id: &str) -> Result<Self, RpcError> {
        let stream = candidates()
            .iter()
            .find_map(|path| Stream::open(path, REPLY_TIMEOUT).ok())
            .ok_or(RpcError::DiscordNotFound)?;
        Self::handshake(stream, application_id)
    }

    pub fn open_at(path: &Path, application_id: &str) -> Result<Self, RpcError> {
        Self::handshake(Stream::open(path, REPLY_TIMEOUT)?, application_id)
    }

    pub fn account(&self) -> Option<&Account> {
        self.account.as_ref()
    }

    pub fn check(&mut self) -> Result<(), RpcError> {
        Frame::new(Opcode::Ping, json!({})).write_to(&mut self.stream)?;
        loop {
            let frame = Frame::read_from(&mut self.stream)?;
            match frame.opcode {
                Opcode::Pong => return Ok(()),
                Opcode::Close => return Err(closed(&frame.body)),
                Opcode::Ping => Frame::new(Opcode::Pong, frame.body).write_to(&mut self.stream)?,
                Opcode::Frame | Opcode::Handshake => {}
            }
        }
    }

    pub fn set_activity(&mut self, activity: &Value) -> Result<Value, RpcError> {
        self.send_activity(activity)
    }

    pub fn clear_activity(&mut self) -> Result<(), RpcError> {
        self.send_activity(&Value::Null).map(drop)
    }

    fn handshake(stream: Stream, application_id: &str) -> Result<Self, RpcError> {
        let mut connection = Self {
            stream,
            next_nonce: 1,
            account: None,
        };
        let hello = json!({ "v": 1, "client_id": application_id });
        Frame::new(Opcode::Handshake, hello).write_to(&mut connection.stream)?;
        let reply = connection.next_message()?;
        match reply["evt"].as_str() {
            Some("READY") => {
                connection.account = Account::from_ready(&reply);
                Ok(connection)
            }
            Some("ERROR") => Err(rejection(&reply)),
            _ => Err(RpcError::Protocol("no READY after handshake")),
        }
    }

    fn send_activity(&mut self, activity: &Value) -> Result<Value, RpcError> {
        let nonce = self.next_nonce.to_string();
        self.next_nonce += 1;
        let command = json!({
            "cmd": "SET_ACTIVITY",
            "args": { "pid": process::id(), "activity": activity },
            "nonce": nonce,
        });
        Frame::new(Opcode::Frame, command).write_to(&mut self.stream)?;
        loop {
            let mut reply = self.next_message()?;
            if reply["nonce"].as_str() != Some(&nonce) {
                continue;
            }
            if reply["evt"].as_str() == Some("ERROR") {
                return Err(rejection(&reply));
            }
            return Ok(reply["data"].take());
        }
    }

    fn next_message(&mut self) -> Result<Value, RpcError> {
        loop {
            let frame = Frame::read_from(&mut self.stream)?;
            match frame.opcode {
                Opcode::Frame => return Ok(frame.body),
                Opcode::Ping => Frame::new(Opcode::Pong, frame.body).write_to(&mut self.stream)?,
                Opcode::Close => return Err(closed(&frame.body)),
                Opcode::Handshake | Opcode::Pong => {}
            }
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = Frame::new(Opcode::Close, json!({})).write_to(&mut self.stream);
    }
}

fn rejection(reply: &Value) -> RpcError {
    RpcError::Rejected {
        code: reply["data"]["code"].as_i64().unwrap_or_default(),
        message: reply["data"]["message"]
            .as_str()
            .unwrap_or("no reason given")
            .to_owned(),
    }
}

fn closed(body: &Value) -> RpcError {
    let reason = body["message"].as_str().unwrap_or("no reason given");
    RpcError::Closed(reason.to_owned())
}
