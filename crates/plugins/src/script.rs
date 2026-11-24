use crate::{
    Plugin,
    host::{Fetch, Host},
    shown::Shown,
};
use flexolotl_core::activity::Activity;
use mlua::{
    ChunkMode, Function, HookTriggers, Lua, LuaOptions, LuaSerdeExt, StdLib, Table, Value, VmState,
};
use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

const MEMORY: usize = 24 * 1024 * 1024;
const LONGEST_RUN: Duration = Duration::from_secs(3);
const CHECK_EVERY: u32 = 5_000;
const PRELUDE: &str = r#"
local set, get, kind, fail = setmetatable, rawget, type, error
setmetatable = function(value, meta)
    if kind(meta) == "table" and get(meta, "__gc") ~= nil then
        fail("not allowed: __gc")
    end
    return set(value, meta)
end
"#;
const FORBIDDEN: [&str; 8] = [
    "load",
    "loadfile",
    "dofile",
    "collectgarbage",
    "print",
    "require",
    "pcall",
    "xpcall",
];
const PAUSE: std::ops::RangeInclusive<f64> = 1. ..=300.;
const USUAL_PAUSE: f64 = 5.;

pub struct Script {
    lua: Lua,
    update: Function,
    name: String,
    pause: Duration,
    deadline: Rc<Cell<Option<Instant>>>,
}

impl Script {
    pub fn start(plugin: &Plugin, fetch: Fetch) -> Result<Self, String> {
        let name = plugin.manifest.name.clone();
        Self::build(plugin, fetch, name).map_err(|error| short(&error))
    }

    pub fn pause(&self) -> Duration {
        self.pause
    }

    pub fn update(&self) -> Result<Option<Activity>, String> {
        self.deadline.set(Some(Instant::now() + LONGEST_RUN));
        let answer = self.update.call::<Value>(());
        self.deadline.set(None);
        let value = answer.map_err(|error| short(&error))?;
        if value.is_nil() {
            return Ok(None);
        }
        let shown: Shown = self.lua.from_value(value).map_err(|error| short(&error))?;
        Ok(shown.into_activity(&self.name))
    }

    fn build(plugin: &Plugin, fetch: Fetch, name: String) -> mlua::Result<Self> {
        let libraries = StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8;
        let lua = Lua::new_with(libraries, LuaOptions::new())?;
        lua.set_memory_limit(MEMORY)?;
        for forbidden in FORBIDDEN {
            lua.globals().set(forbidden, Value::Nil)?;
        }
        lua.load(PRELUDE).exec()?;
        let deadline = Rc::new(Cell::new(Some(Instant::now() + LONGEST_RUN)));
        let watched = deadline.clone();
        lua.set_hook(
            HookTriggers::new().every_nth_instruction(CHECK_EVERY),
            move |_, _| match watched.get() {
                Some(deadline) if Instant::now() > deadline => {
                    Err(mlua::Error::runtime("the plugin works for too long"))
                }
                _ => Ok(VmState::Continue),
            },
        )?;
        Host::new(&plugin.manifest.permissions, fetch).attach(&lua)?;
        let main: Table = lua
            .load(&plugin.script)
            .set_name("=main.lua")
            .set_mode(ChunkMode::Text)
            .eval()?;
        let update = main.get("update")?;
        let pause: f64 = main.get::<Option<f64>>("every")?.unwrap_or(USUAL_PAUSE);
        let pause = if pause.is_finite() {
            pause
        } else {
            USUAL_PAUSE
        };
        deadline.set(None);
        Ok(Self {
            lua,
            update,
            name,
            pause: Duration::from_secs_f64(pause.clamp(*PAUSE.start(), *PAUSE.end())),
            deadline,
        })
    }
}

fn short(error: &mlua::Error) -> String {
    let text = error.to_string();
    let first = text.lines().next().unwrap_or_default();
    first.trim_start_matches("runtime error: ").to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Manifest;
    use std::sync::Arc;

    fn script(source: &str) -> Result<Script, String> {
        let manifest = r#"{"id":"test","name":"Test","version":"1","api":1}"#;
        let plugin = Plugin {
            manifest: Manifest::parse(manifest).unwrap(),
            script: source.to_owned(),
        };
        Script::start(&plugin, Arc::new(|_| None))
    }

    fn run(body: &str) -> Result<Option<Activity>, String> {
        script(&format!("return {{ update = function() {body} end }}"))?.update()
    }

    #[test]
    fn runs_a_plugin() {
        let shown = run("return { title = 'Cave', subtitle = 'dark' }")
            .unwrap()
            .unwrap();
        assert_eq!((&*shown.name, &*shown.details), ("Cave", "dark"));
        assert_eq!(run("return nil").unwrap(), None);
    }

    #[test]
    fn keeps_state_between_updates() {
        let counter = script(
            "local n = 0 return { every = 60, update = function() n = n + 1 return { title = 'n ' .. n } end }",
        )
        .unwrap();
        counter.update().unwrap();
        assert_eq!(counter.update().unwrap().unwrap().name, "n 2");
        assert_eq!(counter.pause(), Duration::from_secs(60));
    }

    #[test]
    fn has_no_way_out() {
        for name in [
            "io",
            "os",
            "package",
            "require",
            "debug",
            "load",
            "loadfile",
            "dofile",
            "coroutine",
            "collectgarbage",
            "pcall",
        ] {
            let shown = run(&format!("return {{ title = type({name}) }}")).unwrap();
            assert_eq!(shown.unwrap().name, "nil", "{name}");
        }
        for call in [
            "read('/etc/passwd')",
            "get('https://a.example')",
            "messages()",
            "running('x')",
        ] {
            assert!(
                run(&format!("return {{ title = flexolotl.{call} }}")).is_err(),
                "{call}"
            );
        }
    }

    #[test]
    fn stops_a_plugin_that_never_ends() {
        let started = Instant::now();
        assert!(run("while true do end").is_err());
        assert!(run("while true do pcall(function() while true do end end) end").is_err());
        assert!(run("setmetatable({}, { __gc = function() while true do end end })").is_err());
        assert!(started.elapsed() < LONGEST_RUN * 4);
    }

    #[test]
    fn stops_a_plugin_that_eats_memory() {
        assert!(run("local t = {} while true do t[#t + 1] = string.rep('x', 100000) end").is_err());
        assert!(run("return { title = string.rep('x', 1000000000) }").is_err());
    }

    #[test]
    fn reports_a_broken_plugin() {
        assert!(script("this is not lua").is_err());
        assert!(script("return 5").is_err());
        assert!(script("return {}").is_err());
        assert!(script("\x1bLua bytecode").is_err());
        assert!(run("return 5").is_err());
    }
}
