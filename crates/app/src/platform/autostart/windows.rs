use crate::BACKGROUND_FLAG;
use std::{env, io, os::windows::process::CommandExt, process::Command};

const KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const NAME: &str = "Flexolotl";
const NO_CONSOLE: u32 = 0x0800_0000;

pub fn is_enabled() -> bool {
    registry(&["query", KEY, "/v", NAME]).is_ok()
}

pub fn set_enabled(enabled: bool) -> io::Result<()> {
    if !enabled {
        return registry(&["delete", KEY, "/v", NAME, "/f"]);
    }
    let program = env::current_exe()?;
    let command = format!("\"{}\" {BACKGROUND_FLAG}", program.display());
    registry(&["add", KEY, "/v", NAME, "/t", "REG_SZ", "/d", &command, "/f"])
}

fn registry(arguments: &[&str]) -> io::Result<()> {
    let status = Command::new("reg.exe")
        .args(arguments)
        .creation_flags(NO_CONSOLE)
        .output()?
        .status;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other("the registry refused the change"))
    }
}
