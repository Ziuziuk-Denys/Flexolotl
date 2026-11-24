use flexolotl_plugins::{Event, Plugin, Runner, Shelf};
use serde_json::{Value, json};
use std::{
    env, fs,
    io::Write,
    net::TcpStream,
    path::{Path, PathBuf},
    sync::Arc,
};

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins")
}

fn plugin(id: &str) -> Plugin {
    Shelf::new(folder()).load(id).unwrap()
}

#[test]
fn every_folder_is_a_plugin_listed_in_the_index() {
    let shelf = Shelf::new(folder());
    let plugins = shelf.plugins();
    let folders = fs::read_dir(folder())
        .unwrap()
        .flatten()
        .filter(|entry| entry.path().is_dir());
    assert_eq!(
        folders.count(),
        plugins.len(),
        "a folder is not a valid plugin"
    );
    let entries: Vec<Value> = plugins
        .iter()
        .map(|plugin| {
            let manifest = &plugin.manifest;
            json!({
                "id": manifest.id,
                "name": manifest.name,
                "about": manifest.about,
                "author": manifest.author,
                "version": manifest.version,
                "translations": manifest.translations,
            })
        })
        .collect();
    let index = serde_json::to_string_pretty(&entries).unwrap() + "\n";
    let path = folder().join("index.json");
    if env::var_os("UPDATE_INDEX").is_some() {
        fs::write(&path, &index).unwrap();
    }
    let known = fs::read_to_string(path).unwrap_or_default();
    assert_eq!(
        known.replace("\r\n", "\n"),
        index,
        "run with UPDATE_INDEX=1"
    );
}

#[test]
fn pomodoro_counts_down() {
    let (_runner, events) = Runner::start(plugin("pomodoro"), Arc::new(|_| None));
    let Ok(Event::Shows(Some(activity))) = events.recv_blocking() else {
        panic!("pomodoro shows a status");
    };
    assert_eq!(activity.name, "Focusing");
}

#[test]
fn dialogs_shows_what_a_game_says() {
    let (_runner, events) = Runner::start(plugin("dialogs"), Arc::new(|_| None));
    assert_eq!(events.recv_blocking(), Ok(Event::Shows(None)));
    let body = r#"{"game":"Deep Cave","speaker":"Mira","line":"Do not look down"}"#;
    let mut stream = TcpStream::connect(("127.0.0.1", 4177)).unwrap();
    let length = body.len();
    write!(
        stream,
        "POST / HTTP/1.1\r\nX-Flexolotl: 1\r\nContent-Length: {length}\r\n\r\n{body}"
    )
    .unwrap();
    let Ok(Event::Shows(Some(activity))) = events.recv_blocking() else {
        panic!("the line is shown");
    };
    assert_eq!(activity.name, "Deep Cave");
    assert_eq!(activity.details, "Mira: Do not look down");
}
