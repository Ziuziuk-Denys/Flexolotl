mod clean;
mod kind;

pub use clean::without_metadata;
pub use kind::MediaKind;

pub const MAX_BYTES: usize = 10 * 1024 * 1024;
