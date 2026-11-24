mod files;
mod messages;
mod network;
mod programs;

pub use network::Fetch;

use crate::Permissions;
use files::Files;
use messages::Messages;
use mlua::{Lua, LuaSerdeExt, Value};
use network::Network;
use std::{
    cell::RefCell,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Host {
    files: Files,
    network: Network,
    messages: Option<Messages>,
    sees_programs: bool,
}

impl Host {
    pub fn new(permissions: &Permissions, fetch: Fetch) -> Self {
        Self {
            files: Files::new(&permissions.files),
            network: Network::new(&permissions.hosts, fetch),
            messages: permissions.port.and_then(Messages::listen),
            sees_programs: permissions.programs,
        }
    }

    pub fn attach(self, lua: &Lua) -> mlua::Result<()> {
        let table = lua.create_table()?;
        let files = std::rc::Rc::new(RefCell::new(self.files));
        let followed = files.clone();
        table.set(
            "follow",
            lua.create_function(move |_, path: String| {
                allowed(followed.borrow_mut().follow(&path), "file", &path)
            })?,
        )?;
        table.set(
            "read",
            lua.create_function(move |_, path: String| {
                allowed(files.borrow().read(&path), "file", &path)
            })?,
        )?;
        let network = self.network;
        table.set(
            "get",
            lua.create_function(move |_, url: String| allowed(network.get(&url), "host", &url))?,
        )?;
        let messages = self.messages;
        table.set(
            "messages",
            lua.create_function(move |_, ()| {
                let taken = messages.as_ref().map(Messages::take);
                allowed(taken, "port", "")
            })?,
        )?;
        let sees_programs = self.sees_programs;
        table.set(
            "running",
            lua.create_function(move |_, name: String| {
                let answer = sees_programs.then(|| programs::is_running(&name));
                allowed(answer, "programs", "")
            })?,
        )?;
        table.set("now", lua.create_function(|_, ()| Ok(now()))?)?;
        table.set(
            "json",
            lua.create_function(|lua, text: String| {
                let value: serde_json::Value = match serde_json::from_str(&text) {
                    Ok(value) => value,
                    Err(_) => return Ok(Value::Nil),
                };
                lua.to_value(&value)
            })?,
        )?;
        lua.globals().set("flexolotl", table)
    }
}

fn allowed<T>(answer: Option<T>, permission: &str, what: &str) -> mlua::Result<T> {
    answer.ok_or_else(|| mlua::Error::runtime(format!("not allowed: {permission} {what}")))
}

fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0., |passed| passed.as_secs_f64())
}
