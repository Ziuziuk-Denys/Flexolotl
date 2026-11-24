mod host;
mod manifest;
mod runner;
mod script;
mod shelf;
mod shown;

pub use host::Fetch;
pub use manifest::{Manifest, Permissions, Words};
pub use runner::{Event, Runner};
pub use shelf::{Plugin, Shelf};

pub const API: u32 = 1;
pub const MANIFEST_FILE: &str = "plugin.json";
pub const SCRIPT_FILE: &str = "main.lua";
