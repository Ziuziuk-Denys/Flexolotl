#[cfg(not(target_os = "linux"))]
mod desktop;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
pub use desktop::TrayIcon;
#[cfg(target_os = "linux")]
pub use linux::TrayIcon;

const ICON_SIDE: u32 = 44;
const ICON: &[u8] = include_bytes!("../../../assets/tray-44.argb");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayState {
    pub is_live: bool,
    pub can_toggle: bool,
    pub title: String,
    pub open: &'static str,
    pub toggle: &'static str,
    pub quit: &'static str,
}

impl TrayState {
    fn description(&self) -> &str {
        if self.is_live { &self.title } else { "" }
    }
}
