use gpui_kit::WindowBackgroundAppearance;

pub const HAS_SYSTEM_CONTROLS: bool = cfg!(target_os = "macos");
pub const HEADER_INSET: f32 = if HAS_SYSTEM_CONTROLS { 88. } else { 24. };
pub const CORNER: f32 = if cfg!(target_os = "linux") { 20. } else { 0. };
pub const BACKGROUND: WindowBackgroundAppearance = if cfg!(target_os = "linux") {
    WindowBackgroundAppearance::Transparent
} else {
    WindowBackgroundAppearance::Opaque
};
