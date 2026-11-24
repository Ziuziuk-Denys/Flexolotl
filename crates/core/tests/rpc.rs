#![cfg(unix)]

use flexolotl_core::rpc::{Connection, Frame, Opcode, RpcError};
use serde_json::{Value, json};
use std::{
    env, fs,
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    process,
    thread::{self, JoinHandle},
};

struct FakeDiscord {
    path: PathBuf,
    server: Option<JoinHandle<()>>,
}

impl FakeDiscord {
    fn start(name: &str, script: impl FnOnce(UnixStream) + Send + 'static) -> Self {
        let path = env::temp_dir().join(format!("flexolotl-{}-{name}", process::id()));
        let listener = UnixListener::bind(&path).unwrap();
        let server = thread::spawn(move || script(listener.accept().unwrap().0));
        Self {
            path,
            server: Some(server),
        }
    }

    fn connect(&self) -> Result<Connection, RpcError> {
        Connection::open_at(&self.path, "42")
    }

    fn finish(mut self) {
        self.server.take().unwrap().join().unwrap();
    }
}

impl Drop for FakeDiscord {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn receive(stream: &mut UnixStream) -> Frame {
    Frame::read_from(stream).unwrap()
}

fn send(stream: &mut UnixStream, opcode: Opcode, body: Value) {
    Frame::new(opcode, body).write_to(stream).unwrap();
}

fn greet(stream: &mut UnixStream) {
    let hello = receive(stream);
    assert_eq!(hello.opcode, Opcode::Handshake);
    assert_eq!(hello.body, json!({ "v": 1, "client_id": "42" }));
    send(stream, Opcode::Frame, json!({ "evt": "READY" }));
}

#[test]
fn sets_an_activity_through_pings_and_unrelated_messages() {
    let discord = FakeDiscord::start("set", |mut stream| {
        greet(&mut stream);
        let command = receive(&mut stream).body;
        assert_eq!(command["cmd"], "SET_ACTIVITY");
        assert_eq!(command["args"]["activity"]["details"], "hi");
        send(&mut stream, Opcode::Ping, json!({ "n": 1 }));
        assert_eq!(receive(&mut stream).opcode, Opcode::Pong);
        send(&mut stream, Opcode::Frame, json!({ "nonce": "other" }));
        let reply = json!({ "nonce": command["nonce"], "data": { "name": "kept" } });
        send(&mut stream, Opcode::Frame, reply);
        let clear = receive(&mut stream).body;
        assert_eq!(clear["args"]["activity"], Value::Null);
        send(
            &mut stream,
            Opcode::Frame,
            json!({ "nonce": clear["nonce"], "data": null }),
        );
        assert_eq!(receive(&mut stream).opcode, Opcode::Close);
    });
    let mut connection = discord.connect().unwrap();
    let data = connection
        .set_activity(&json!({ "details": "hi" }))
        .unwrap();
    assert_eq!(data["name"], "kept");
    connection.clear_activity().unwrap();
    drop(connection);
    discord.finish();
}

#[test]
fn reports_a_rejected_activity() {
    let discord = FakeDiscord::start("reject", |mut stream| {
        greet(&mut stream);
        let nonce = receive(&mut stream).body["nonce"].clone();
        let data = json!({ "code": 4000, "message": "bad button" });
        send(
            &mut stream,
            Opcode::Frame,
            json!({ "nonce": nonce, "evt": "ERROR", "data": data }),
        );
        receive(&mut stream);
    });
    let mut connection = discord.connect().unwrap();
    let error = connection.set_activity(&json!({})).unwrap_err();
    assert!(
        matches!(error, RpcError::Rejected { code: 4000, ref message } if message == "bad button")
    );
    drop(connection);
    discord.finish();
}

#[test]
fn reports_why_discord_closed_the_connection() {
    let discord = FakeDiscord::start("close", |mut stream| {
        receive(&mut stream);
        send(
            &mut stream,
            Opcode::Close,
            json!({ "message": "Invalid Client ID" }),
        );
    });
    let error = discord.connect().unwrap_err();
    assert!(matches!(error, RpcError::Closed(ref reason) if reason == "Invalid Client ID"));
    discord.finish();
}

#[test]
fn refuses_an_oversized_frame() {
    let mut bytes = Vec::new();
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(u32::MAX.to_le_bytes());
    let error = Frame::read_from(&mut bytes.as_slice()).unwrap_err();
    assert!(matches!(error, RpcError::Protocol(_)));
}

#[test]
fn reads_the_account_and_checks_the_connection() {
    let discord = FakeDiscord::start("account", |mut stream| {
        receive(&mut stream);
        let user = json!({
            "id": "7",
            "username": "axo",
            "global_name": null,
            "avatar": "ff",
            "avatar_decoration_data": { "asset": "a_1" },
        });
        let ready = json!({ "evt": "READY", "data": { "user": user } });
        send(&mut stream, Opcode::Frame, ready);
        assert_eq!(receive(&mut stream).opcode, Opcode::Ping);
        send(&mut stream, Opcode::Pong, json!({}));
        receive(&mut stream);
    });
    let mut connection = discord.connect().unwrap();
    let account = connection.account().unwrap();
    assert_eq!(account.display_name, "axo");
    assert_eq!(
        account.avatar.as_deref(),
        Some("https://cdn.discordapp.com/avatars/7/ff.png?size=160")
    );
    assert!(account.decoration.as_deref().unwrap().contains("/a_1.png"));
    connection.check().unwrap();
    drop(connection);
    discord.finish();
}
