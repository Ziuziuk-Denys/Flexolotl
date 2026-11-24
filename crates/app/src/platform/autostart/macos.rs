use crate::{BACKGROUND_FLAG, platform::paths};
use std::{env, fs, io, path::PathBuf};

const LABEL: &str = "quest.nuvo.Flexolotl";

pub fn is_enabled() -> bool {
    file().exists()
}

pub fn set_enabled(enabled: bool) -> io::Result<()> {
    if !enabled {
        return fs::remove_file(file());
    }
    let program = env::current_exe()?;
    let program = program
        .to_string_lossy()
        .replace('&', "&amp;")
        .replace('<', "&lt;");
    let agent = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\"><dict><key>Label</key><string>{LABEL}</string>\
         <key>ProgramArguments</key><array><string>{program}</string>\
         <string>{BACKGROUND_FLAG}</string></array>\
         <key>RunAtLoad</key><true/></dict></plist>\n"
    );
    fs::create_dir_all(directory())?;
    fs::write(file(), agent)
}

fn directory() -> PathBuf {
    paths::home().join("Library/LaunchAgents")
}

fn file() -> PathBuf {
    directory().join(format!("{LABEL}.plist"))
}
