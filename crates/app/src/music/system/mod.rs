#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::Media;
#[cfg(target_os = "macos")]
pub use macos::Media;
#[cfg(target_os = "windows")]
pub use windows::Media;

const BROWSERS: [&str; 16] = [
    "chromium",
    "chrome",
    "google-chrome",
    "firefox",
    "brave",
    "msedge",
    "microsoft-edge",
    "microsoftedge",
    "vivaldi",
    "opera",
    "zen",
    "librewolf",
    "waterfox",
    "thorium",
    "arc",
    "plasma-browser",
];

pub fn is_browser(program: &str) -> bool {
    let program = program.to_lowercase();
    BROWSERS
        .iter()
        .any(|browser| program == *browser || program.starts_with(&format!("{browser}-")))
        || program
            .strip_suffix(".exe")
            .is_some_and(|name| BROWSERS.contains(&name))
}

#[cfg(test)]
mod tests {
    use super::is_browser;

    #[test]
    fn tells_a_browser_from_an_app_built_on_one() {
        for browser in [
            "chromium-browser",
            "chrome",
            "firefox",
            "msedge.exe",
            "Brave",
        ] {
            assert!(is_browser(browser), "{browser}");
        }
        for app in ["nuvo", "spotify", "Nuvo.exe", "vlc", "zenity"] {
            assert!(!is_browser(app), "{app}");
        }
    }
}
