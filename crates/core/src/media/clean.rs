use super::MediaKind;

pub fn without_metadata(bytes: &[u8]) -> Option<Vec<u8>> {
    match MediaKind::of(bytes)? {
        MediaKind::Gif => Some(bytes.to_vec()),
        MediaKind::Png => clean_png(bytes),
        MediaKind::Jpeg => clean_jpeg(bytes),
        MediaKind::Webp => clean_webp(bytes),
    }
}

fn clean_png(bytes: &[u8]) -> Option<Vec<u8>> {
    const PRIVATE: [&[u8]; 5] = [b"eXIf", b"tEXt", b"zTXt", b"iTXt", b"tIME"];
    let (signature, mut rest) = bytes.split_at_checked(8)?;
    let mut clean = signature.to_vec();
    while !rest.is_empty() {
        let length = u32::from_be_bytes(rest.get(..4)?.try_into().ok()?) as usize;
        let (chunk, after) = rest.split_at_checked(length.checked_add(12)?)?;
        if !PRIVATE.contains(&&chunk[4..8]) {
            clean.extend_from_slice(chunk);
        }
        if &chunk[4..8] == b"IEND" {
            break;
        }
        rest = after;
    }
    Some(clean)
}

fn clean_jpeg(bytes: &[u8]) -> Option<Vec<u8>> {
    const START_OF_SCAN: u8 = 0xda;
    let mut clean = bytes.get(..2)?.to_vec();
    let mut position = 2;
    loop {
        let &[marker_start, marker] = bytes.get(position..position + 2)?.as_array()?;
        if marker_start != 0xff {
            return None;
        }
        if marker == 0xff {
            position += 1;
            continue;
        }
        if marker == START_OF_SCAN {
            clean.extend_from_slice(&bytes[position..]);
            return Some(clean);
        }
        let length = u16::from_be_bytes(bytes.get(position + 2..position + 4)?.try_into().ok()?);
        let end = position + 2 + usize::from(length);
        let segment = bytes.get(position..end)?;
        if !is_private_jpeg(marker) {
            clean.extend_from_slice(segment);
        }
        position = end;
    }
}

fn is_private_jpeg(marker: u8) -> bool {
    const ICC_PROFILE: u8 = 0xe2;
    const ADOBE_COLOR: u8 = 0xee;
    const COMMENT: u8 = 0xfe;
    let is_application = (0xe1..=0xef).contains(&marker);
    marker == COMMENT || is_application && marker != ICC_PROFILE && marker != ADOBE_COLOR
}

fn clean_webp(bytes: &[u8]) -> Option<Vec<u8>> {
    const EXIF_AND_XMP_FLAGS: u8 = 0b0000_1100;
    let mut clean = bytes.get(..12)?.to_vec();
    let mut rest = &bytes[12..];
    while rest.len() >= 8 {
        let length = u32::from_le_bytes(rest[4..8].try_into().ok()?) as usize;
        let padded = length.checked_add(length % 2)?.checked_add(8)?;
        let (chunk, after) = rest.split_at_checked(padded.min(rest.len()))?;
        let name = &chunk[..4];
        if name != b"EXIF" && name != b"XMP " {
            let start = clean.len();
            clean.extend_from_slice(chunk);
            if name == b"VP8X" {
                *clean.get_mut(start + 8)? &= !EXIF_AND_XMP_FLAGS;
            }
        }
        rest = after;
    }
    let size = u32::try_from(clean.len().checked_sub(8)?).ok()?;
    clean[4..8].copy_from_slice(&size.to_le_bytes());
    Some(clean)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_chunk(name: &[u8], data: &[u8]) -> Vec<u8> {
        let mut chunk = (data.len() as u32).to_be_bytes().to_vec();
        chunk.extend_from_slice(name);
        chunk.extend_from_slice(data);
        chunk.extend_from_slice(&[0; 4]);
        chunk
    }

    fn webp_chunk(name: &[u8], data: &[u8]) -> Vec<u8> {
        let mut chunk = name.to_vec();
        chunk.extend_from_slice(&(data.len() as u32).to_le_bytes());
        chunk.extend_from_slice(data);
        chunk.resize(chunk.len() + data.len() % 2, 0);
        chunk
    }

    fn contains(bytes: &[u8], needle: &[u8]) -> bool {
        bytes.windows(needle.len()).any(|window| window == needle)
    }

    #[test]
    fn removes_the_place_from_a_png() {
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        for (name, data) in [
            (&b"IHDR"[..], &b"size"[..]),
            (b"eXIf", b"GPS"),
            (b"IDAT", b"pixels"),
            (b"IEND", b""),
        ] {
            png.extend(png_chunk(name, data));
        }
        let clean = without_metadata(&png).unwrap();
        assert!(!contains(&clean, b"GPS"));
        assert!(contains(&clean, b"pixels") && contains(&clean, b"IEND"));
    }

    #[test]
    fn removes_the_place_from_a_jpeg_and_keeps_its_colors() {
        let mut jpeg = vec![0xff, 0xd8];
        for (marker, data) in [
            (0xe0, &b"JFIF"[..]),
            (0xe1, b"GPS"),
            (0xe2, b"ICC"),
            (0xfe, b"note"),
            (0xdb, b"table"),
        ] {
            jpeg.extend_from_slice(&[0xff, marker, 0, data.len() as u8 + 2]);
            jpeg.extend_from_slice(data);
        }
        jpeg.extend_from_slice(&[0xff, 0xda, 1, 2, 3, 0xff, 0xd9]);
        let clean = without_metadata(&jpeg).unwrap();
        assert!(!contains(&clean, b"GPS") && !contains(&clean, b"note"));
        assert!(contains(&clean, b"ICC") && contains(&clean, b"table"));
        assert!(clean.ends_with(&[0xff, 0xda, 1, 2, 3, 0xff, 0xd9]));
    }

    #[test]
    fn removes_the_place_from_a_webp() {
        let mut webp = b"RIFF\0\0\0\0WEBP".to_vec();
        webp.extend(webp_chunk(b"VP8X", &[0b0010_1100, 0, 0, 0]));
        webp.extend(webp_chunk(b"VP8 ", b"pixels"));
        webp.extend(webp_chunk(b"EXIF", b"GPS"));
        let clean = without_metadata(&webp).unwrap();
        assert!(!contains(&clean, b"GPS") && contains(&clean, b"pixels"));
        assert_eq!(clean[20], 0b0010_0000);
        let size = u32::from_le_bytes(clean[4..8].try_into().unwrap()) as usize;
        assert_eq!(size, clean.len() - 8);
    }

    #[test]
    fn refuses_damaged_and_foreign_files() {
        assert_eq!(without_metadata(b"<svg></svg>"), None);
        assert_eq!(without_metadata(b"\x89PNG\r\n\x1a\n\xff\xff\xff\xff"), None);
        assert_eq!(without_metadata(&[0xff, 0xd8, 0xff, 0xe1, 0xff]), None);
    }
}
