use std::{
    collections::HashMap,
    env, fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

const LARGEST: u64 = 1024 * 1024;
const FIRST_LOOK: u64 = 64 * 1024;

pub struct Files {
    allowed: Vec<(String, Option<PathBuf>)>,
    followed: HashMap<PathBuf, u64>,
}

impl Files {
    pub fn new(patterns: &[String]) -> Self {
        let allowed = patterns
            .iter()
            .map(|pattern| (pattern.clone(), expand(pattern)))
            .collect();
        Self {
            allowed,
            followed: HashMap::new(),
        }
    }

    pub fn read(&self, path: &str) -> Option<Option<String>> {
        let path = self.resolve(path)?;
        Some(path.and_then(|path| Some(read_from(&path, 0)?.0)))
    }

    pub fn follow(&mut self, path: &str) -> Option<Vec<String>> {
        let Some(path) = self.resolve(path)? else {
            return Some(Vec::new());
        };
        let size = fs::metadata(&path).map_or(0, |data| data.len());
        let known = self.followed.get(&path).copied();
        let start = match known {
            Some(offset) if offset <= size => offset,
            Some(_) => 0,
            None => size.saturating_sub(FIRST_LOOK),
        };
        let Some((text, read)) = read_from(&path, start) else {
            return Some(Vec::new());
        };
        let complete = text.rfind('\n').map_or(0, |end| end + 1);
        let used = if read == LARGEST && complete == 0 {
            read
        } else {
            text[..complete].len() as u64
        };
        self.followed.insert(path, start + used);
        Some(text[..complete].lines().map(str::to_owned).collect())
    }

    fn resolve(&self, asked: &str) -> Option<Option<PathBuf>> {
        if asked.split(['/', '\\']).any(|part| part == "..") {
            return None;
        }
        self.allowed.iter().find_map(|(pattern, path)| {
            if pattern == asked {
                return Some(path.clone());
            }
            let inside = asked.strip_prefix(pattern.as_str())?;
            pattern
                .ends_with('/')
                .then(|| inside_folder(path.as_deref()?, inside))
        })
    }
}

fn inside_folder(folder: &Path, inside: &str) -> Option<PathBuf> {
    let real = fs::canonicalize(folder.join(inside)).ok()?;
    let root = fs::canonicalize(folder).ok()?;
    real.starts_with(root).then_some(real)
}

fn read_from(path: &Path, start: u64) -> Option<(String, u64)> {
    let mut file = fs::File::open(path).ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut bytes = Vec::new();
    file.take(LARGEST).read_to_end(&mut bytes).ok()?;
    let read = bytes.len() as u64;
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => {
            let valid = error.utf8_error().valid_up_to();
            let bytes = error.into_bytes();
            String::from_utf8(bytes[..valid].to_vec()).ok()?
        }
    };
    Some((text, read))
}

fn expand(pattern: &str) -> Option<PathBuf> {
    if let Some(rest) = pattern.strip_prefix("~/") {
        return Some(env::home_dir()?.join(rest));
    }
    for variable in ["APPDATA", "LOCALAPPDATA"] {
        if let Some(rest) = pattern.strip_prefix(&format!("%{variable}%/")) {
            return Some(PathBuf::from(env::var_os(variable)?).join(rest));
        }
    }
    Some(PathBuf::from(pattern))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, process};

    fn folder(name: &str) -> PathBuf {
        let folder = env::temp_dir().join(format!("flexolotl-files-{name}-{}", process::id()));
        fs::create_dir_all(&folder).unwrap();
        folder
    }

    #[test]
    fn follows_only_new_whole_lines() {
        let folder = folder("follow");
        let log = folder.join("game.log");
        let name = log.to_string_lossy().into_owned();
        fs::write(&log, "one\ntwo\nthr").unwrap();
        let mut files = Files::new(std::slice::from_ref(&name));
        assert_eq!(files.follow(&name).unwrap(), ["one", "two"]);
        assert!(files.follow(&name).unwrap().is_empty());
        let mut file = fs::OpenOptions::new().append(true).open(&log).unwrap();
        file.write_all("ee\nчетыре\n".as_bytes()).unwrap();
        assert_eq!(files.follow(&name).unwrap(), ["three", "четыре"]);
        fs::write(&log, "new\n").unwrap();
        assert_eq!(files.follow(&name).unwrap(), ["new"]);
        fs::remove_dir_all(folder).unwrap();
    }

    #[cfg(not(windows))]
    #[test]
    fn finds_nothing_where_another_system_keeps_its_files() {
        let path = "%APPDATA%/.game/log.txt".to_owned();
        let mut files = Files::new(std::slice::from_ref(&path));
        assert_eq!(files.read(&path), Some(None));
        assert_eq!(files.follow(&path), Some(Vec::new()));
    }

    #[test]
    fn stays_inside_what_was_allowed() {
        let folder = folder("inside");
        let (open, shut) = (folder.join("open"), folder.join("shut"));
        fs::create_dir_all(&open).unwrap();
        fs::create_dir_all(&shut).unwrap();
        fs::write(open.join("a.txt"), "a").unwrap();
        fs::write(shut.join("secret.txt"), "b").unwrap();
        let root = format!("{}/", open.to_string_lossy());
        let files = Files::new(std::slice::from_ref(&root));
        assert_eq!(files.read(&format!("{root}a.txt")), Some(Some("a".into())));
        assert_eq!(files.read(&format!("{root}../shut/secret.txt")), None);
        assert_eq!(files.read(&shut.join("secret.txt").to_string_lossy()), None);
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(shut.join("secret.txt"), open.join("link")).unwrap();
            assert_eq!(files.read(&format!("{root}link")), Some(None));
        }
        fs::remove_dir_all(folder).unwrap();
    }
}
