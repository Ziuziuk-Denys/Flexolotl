use crate::{BACKGROUND_FLAG, platform::paths};
use std::{env, fs, io, path::PathBuf};

pub fn is_enabled() -> bool {
    file().exists()
}

pub fn set_enabled(enabled: bool) -> io::Result<()> {
    if !enabled {
        return fs::remove_file(file());
    }
    let program = env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .map_or_else(env::current_exe, Ok)?;
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=Flexolotl\nExec=\"{}\" {BACKGROUND_FLAG}\n",
        program.display()
    );
    fs::create_dir_all(directory())?;
    fs::write(file(), entry)
}

fn directory() -> PathBuf {
    paths::config_home().join("autostart")
}

fn file() -> PathBuf {
    directory().join("quest.nuvo.Flexolotl.desktop")
}
