#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub fn give_back() {
    linux::give_back();
}

#[cfg(target_os = "windows")]
pub fn give_back() {
    use windows::Win32::System::{ProcessStatus::EmptyWorkingSet, Threading::GetCurrentProcess};
    let _ = unsafe { EmptyWorkingSet(GetCurrentProcess()) };
}

#[cfg(not(any(all(target_os = "linux", target_env = "gnu"), target_os = "windows")))]
pub fn give_back() {}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
mod linux {
    use std::{ffi::c_void, fs, ops::Range, os::unix::ffi::OsStrExt};

    const DONT_NEED: i32 = 4;

    unsafe extern "C" {
        fn malloc_trim(keep: usize) -> i32;
        fn madvise(start: *mut c_void, length: usize, advice: i32) -> i32;
    }

    pub fn give_back() {
        unsafe { malloc_trim(0) };
        let (Ok(maps), Ok(program)) = (
            fs::read_to_string("/proc/self/maps"),
            fs::read_link("/proc/self/exe"),
        ) else {
            return;
        };
        for range in unchanged_parts(&maps, program.as_os_str().as_bytes()) {
            let start = std::ptr::with_exposed_provenance_mut(range.start);
            unsafe { madvise(start, range.len(), DONT_NEED) };
        }
    }

    fn unchanged_parts(maps: &str, program: &[u8]) -> Vec<Range<usize>> {
        let parts: Vec<_> = maps
            .lines()
            .filter_map(|line| part_of(line, program))
            .collect();
        let Some(code) = parts.iter().position(|(rights, _)| *rights == "r-xp") else {
            return Vec::new();
        };
        parts[..=code]
            .iter()
            .filter(|(rights, _)| matches!(*rights, "r--p" | "r-xp"))
            .map(|(_, range)| range.clone())
            .collect()
    }

    fn part_of<'a>(line: &'a str, program: &[u8]) -> Option<(&'a str, Range<usize>)> {
        let mut parts = line.split_ascii_whitespace();
        let (range, rights) = (parts.next()?, parts.next()?);
        if parts.nth(3)?.as_bytes() != program {
            return None;
        }
        let (start, end) = range.split_once('-')?;
        let read = |text| usize::from_str_radix(text, 16).ok();
        Some((rights, read(start)?..read(end)?))
    }

    #[cfg(test)]
    mod tests {
        use super::unchanged_parts;

        #[test]
        fn leaves_everything_that_was_written_to() {
            let maps = "1000-2000 r--p 00000000 00:1f 42   /opt/flexolotl\n\
                        2000-5000 r-xp 00001000 00:1f 42   /opt/flexolotl\n\
                        5000-6000 r--p 00004000 00:1f 42   /opt/flexolotl\n\
                        6000-7000 rw-p 00005000 00:1f 42   /opt/flexolotl\n\
                        8000-9000 r-xp 00000000 00:1f 43   /usr/lib64/libc.so.6\n\
                        9000-a000 r-xp 00000000 00:00 0    [vdso]";
            assert_eq!(
                unchanged_parts(maps, b"/opt/flexolotl"),
                [0x1000..0x2000, 0x2000..0x5000]
            );
            assert!(unchanged_parts(maps, b"/opt/other").is_empty());
        }
    }
}
