mod clean;
mod kind;

pub use clean::without_metadata;
pub use kind::MediaKind;

pub fn is_animated(bytes: &[u8], kind: MediaKind) -> bool {
    let has = |mark: &[u8]| bytes.windows(mark.len()).any(|part| part == mark);
    match kind {
        MediaKind::Gif => true,
        MediaKind::Png => has(b"acTL"),
        MediaKind::Webp => has(b"ANIM"),
        MediaKind::Jpeg => false,
    }
}

pub const MAX_BYTES: usize = 10 * 1024 * 1024;
