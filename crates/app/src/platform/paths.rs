use std::{env, path::PathBuf};

const NAME: &str = "flexolotl";

pub fn config_directory() -> PathBuf {
    config_home().join(NAME)
}

pub fn instance_file() -> PathBuf {
    config_directory().join("instance")
}

#[cfg(target_os = "linux")]
pub fn config_home() -> PathBuf {
    absolute("XDG_CONFIG_HOME").unwrap_or_else(|| home().join(".config"))
}

#[cfg(target_os = "macos")]
pub fn config_home() -> PathBuf {
    home().join("Library/Application Support")
}

#[cfg(target_os = "windows")]
pub fn config_home() -> PathBuf {
    absolute("APPDATA").unwrap_or_else(|| home().join("AppData/Roaming"))
}

pub fn home() -> PathBuf {
    env::home_dir().unwrap_or_else(env::temp_dir)
}

#[cfg(not(target_os = "macos"))]
fn absolute(variable: &str) -> Option<PathBuf> {
    env::var_os(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}
