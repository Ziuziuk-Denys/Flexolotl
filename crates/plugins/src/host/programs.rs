#[cfg(target_os = "linux")]
pub fn is_running(name: &str) -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    entries.flatten().any(|entry| {
        let program = std::fs::read_link(entry.path().join("exe"));
        program.is_ok_and(|program| {
            let file = program.file_name().unwrap_or_default().to_string_lossy();
            is_same(&file, name)
        })
    })
}

#[cfg(not(target_os = "linux"))]
pub fn is_running(name: &str) -> bool {
    let mut command = if cfg!(windows) {
        let mut list = std::process::Command::new("tasklist");
        list.args(["/fo", "csv", "/nh"]);
        list
    } else {
        let mut list = std::process::Command::new("ps");
        list.args(["-axco", "comm="]);
        list
    };
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut command, 0x0800_0000);
    let Ok(output) = command.output() else {
        return false;
    };
    String::from_utf8_lossy(&output.stdout).lines().any(|line| {
        let program = line.split("\",\"").next().unwrap_or_default();
        is_same(program.trim_matches('"'), name)
    })
}

fn is_same(program: &str, name: &str) -> bool {
    let program = program.to_lowercase();
    let name = name.to_lowercase();
    program == name || program.strip_suffix(".exe") == Some(name.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_names_with_and_without_the_ending() {
        assert!(is_same("Minecraft.exe", "minecraft") && is_same("java", "Java"));
        assert!(!is_same("javaw", "java") && !is_running("no-such-program-at-all"));
    }
}
