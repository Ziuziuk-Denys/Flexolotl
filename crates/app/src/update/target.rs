use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(target_arch = "x86_64")]
const MACHINE: [&str; 2] = ["x86_64", "x64"];
#[cfg(target_arch = "aarch64")]
const MACHINE: [&str; 2] = ["arm64", "arm64"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub installed: PathBuf,
    pub launched: PathBuf,
    pub suffix: String,
    inside_archive: Option<&'static str>,
}

impl Target {
    pub fn find() -> Option<Self> {
        let target = Self::of_this_system(env::current_exe().ok()?)?;
        let is_development = target.installed.with_file_name("deps").is_dir();
        (!is_development).then_some(target)
    }

    pub fn is_replaceable(&self) -> bool {
        let is_moved_by_system = self
            .installed
            .to_string_lossy()
            .contains("/AppTranslocation/");
        !is_moved_by_system && self.installed.parent().is_some_and(is_writable)
    }

    pub fn install(&self, download: &Path) -> io::Result<()> {
        let staged = self.beside("new");
        remove(&staged)?;
        match self.inside_archive {
            None => {
                fs::copy(download, &staged)?;
            }
            Some(inner) => {
                let unpacked = self.beside("unpacked");
                remove(&unpacked)?;
                fs::create_dir_all(&unpacked)?;
                let moved = unpack(download, &unpacked)
                    .and_then(|()| fs::rename(unpacked.join(inner), &staged));
                remove(&unpacked)?;
                moved?;
            }
        }
        make_executable(&staged)?;
        swap(&self.installed, &staged, &self.beside("old"))
    }

    pub fn clean_up(&self) {
        for leftover in ["old", "new", "unpacked"] {
            let _ = remove(&self.beside(leftover));
        }
    }

    pub fn download_path(&self) -> PathBuf {
        self.beside("download")
    }

    #[cfg(target_os = "linux")]
    fn of_this_system(program: PathBuf) -> Option<Self> {
        let machine = MACHINE[0];
        Some(match env::var_os("APPIMAGE").map(PathBuf::from) {
            Some(image) => Self {
                installed: image.clone(),
                launched: image,
                suffix: format!("linux-{machine}.AppImage"),
                inside_archive: None,
            },
            None => Self {
                installed: program.clone(),
                launched: program,
                suffix: format!("linux-{machine}.tar.gz"),
                inside_archive: Some("usr/bin/flexolotl"),
            },
        })
    }

    #[cfg(target_os = "macos")]
    fn of_this_system(program: PathBuf) -> Option<Self> {
        let bundle = program.ancestors().nth(3)?.to_owned();
        let is_bundle = bundle.extension().is_some_and(|kind| kind == "app");
        is_bundle.then(|| Self {
            installed: bundle,
            launched: program,
            suffix: format!("mac-{}.zip", MACHINE[1]),
            inside_archive: Some("Flexolotl.app"),
        })
    }

    #[cfg(target_os = "windows")]
    fn of_this_system(program: PathBuf) -> Option<Self> {
        Some(Self {
            installed: program.clone(),
            launched: program,
            suffix: format!("win-{}.zip", MACHINE[1]),
            inside_archive: Some("flexolotl.exe"),
        })
    }

    fn beside(&self, ending: &str) -> PathBuf {
        let mut name = self.installed.file_name().unwrap_or_default().to_owned();
        name.push(".");
        name.push(ending);
        self.installed.with_file_name(name)
    }
}

fn swap(installed: &Path, staged: &Path, old: &Path) -> io::Result<()> {
    remove(old)?;
    fs::rename(installed, old)?;
    if let Err(error) = fs::rename(staged, installed) {
        fs::rename(old, installed)?;
        return Err(error);
    }
    Ok(())
}

fn unpack(archive: &Path, directory: &Path) -> io::Result<()> {
    let mut command = if cfg!(target_os = "macos") {
        let mut ditto = Command::new("ditto");
        ditto.args(["-x", "-k"]).arg(archive).arg(directory);
        ditto
    } else {
        let mut tar = Command::new("tar");
        tar.arg("-xf").arg(archive).arg("-C").arg(directory);
        tar
    };
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut command, 0x0800_0000);
    if command.status()?.success() {
        Ok(())
    } else {
        Err(io::Error::other("the archive cannot be unpacked"))
    }
}

fn remove(path: &Path) -> io::Result<()> {
    let removed = if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    match removed {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

fn is_writable(directory: &Path) -> bool {
    let probe = directory.join(format!(".flexolotl-{}", std::process::id()));
    let is_writable = fs::write(&probe, []).is_ok();
    let _ = fs::remove_file(probe);
    is_writable
}

#[cfg(unix)]
fn make_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if path.is_file() {
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(name: &str) -> Target {
        let directory = env::temp_dir().join(format!("flexolotl-{name}-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let installed = directory.join("program");
        fs::write(&installed, "old").unwrap();
        Target {
            launched: installed.clone(),
            installed,
            suffix: String::new(),
            inside_archive: None,
        }
    }

    #[test]
    fn replaces_the_program_and_cleans_up_after_itself() {
        let target = target("swap");
        let download = target.download_path();
        fs::write(&download, "new").unwrap();
        target.install(&download).unwrap();
        assert_eq!(fs::read_to_string(&target.installed).unwrap(), "new");
        assert_eq!(fs::read_to_string(target.beside("old")).unwrap(), "old");
        target.clean_up();
        assert!(!target.beside("old").exists());
        fs::remove_dir_all(target.installed.parent().unwrap()).unwrap();
    }

    #[test]
    fn keeps_the_program_when_the_new_one_is_missing() {
        let target = target("missing");
        assert!(target.install(Path::new("/nowhere/at/all")).is_err());
        let old = target.beside("old");
        assert!(swap(&target.installed, Path::new("/nowhere/at/all"), &old).is_err());
        assert_eq!(fs::read_to_string(&target.installed).unwrap(), "old");
        fs::remove_dir_all(target.installed.parent().unwrap()).unwrap();
    }
}
