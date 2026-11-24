#[cfg(unix)]
pub use unix::candidates;

#[cfg(windows)]
pub fn candidates() -> Vec<std::path::PathBuf> {
    (0..SLOTS)
        .map(|slot| format!(r"\\?\pipe\discord-ipc-{slot}").into())
        .collect()
}

const SLOTS: u8 = 10;

#[cfg(unix)]
mod unix {
    use super::SLOTS;
    use std::{env, ffi::OsString, path::PathBuf};

    const SANDBOX_DIRECTORIES: [&str; 4] = [
        "app/com.discordapp.Discord",
        "app/com.discordapp.DiscordCanary",
        "app/dev.vencord.Vesktop",
        "snap.discord",
    ];
    const ROOT_VARIABLES: [&str; 4] = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"];

    pub fn candidates() -> Vec<PathBuf> {
        candidates_in(|name| env::var_os(name))
    }

    fn candidates_in(variable: impl Fn(&str) -> Option<OsString>) -> Vec<PathBuf> {
        let mut roots: Vec<PathBuf> = ROOT_VARIABLES
            .iter()
            .filter_map(|name| variable(name))
            .map(PathBuf::from)
            .collect();
        roots.push(PathBuf::from("/tmp"));
        let mut seen = Vec::new();
        roots.retain(|root| {
            let is_new = !seen.contains(root);
            seen.push(root.clone());
            is_new
        });
        roots
            .iter()
            .flat_map(|root| {
                let sandboxed = SANDBOX_DIRECTORIES.map(|directory| root.join(directory));
                std::iter::once(root.clone()).chain(sandboxed)
            })
            .flat_map(|directory| {
                (0..SLOTS).map(move |slot| directory.join(format!("discord-ipc-{slot}")))
            })
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn looks_inside_the_flatpak_directory() {
            let paths =
                candidates_in(|name| (name == "XDG_RUNTIME_DIR").then(|| "/run/user/1".into()));
            let expected = PathBuf::from("/run/user/1/app/com.discordapp.Discord/discord-ipc-0");
            assert!(paths.contains(&expected));
            assert_eq!(paths[0], PathBuf::from("/run/user/1/discord-ipc-0"));
        }

        #[test]
        fn does_not_repeat_a_shared_root() {
            let paths = candidates_in(|_| Some("/tmp".into()));
            assert_eq!(
                paths.len(),
                usize::from(SLOTS) * (SANDBOX_DIRECTORIES.len() + 1)
            );
        }
    }
}
