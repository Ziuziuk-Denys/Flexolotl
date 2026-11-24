use flexolotl_core::media::{self, MediaKind};
use image::{
    DynamicImage, ImageDecoder, ImageFormat, ImageReader,
    codecs::{jpeg::JpegEncoder, webp::WebPEncoder},
    imageops::FilterType,
};
use std::io::Cursor;

const SIDE: u32 = 512;
const QUALITY: u8 = 86;
const MOST_PIXELS_PER_SIDE: u32 = 16_384;

pub fn prepare(bytes: &[u8], kind: MediaKind) -> Option<(Vec<u8>, MediaKind)> {
    let clean = media::without_metadata(bytes)?;
    let small = (!media::is_animated(bytes, kind))
        .then(|| shrink(bytes, kind))
        .flatten();
    match small {
        Some(small) => Some(small),
        None => Some((clean, kind)),
    }
}

fn shrink(bytes: &[u8], kind: MediaKind) -> Option<(Vec<u8>, MediaKind)> {
    let format = match kind {
        MediaKind::Png => ImageFormat::Png,
        MediaKind::Jpeg => ImageFormat::Jpeg,
        MediaKind::Webp => ImageFormat::WebP,
        MediaKind::Gif => return None,
    };
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MOST_PIXELS_PER_SIDE);
    limits.max_image_height = Some(MOST_PIXELS_PER_SIDE);
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().ok()?;
    let orientation = decoder.orientation().ok()?;
    let mut picture = DynamicImage::from_decoder(decoder).ok()?;
    picture.apply_orientation(orientation);
    if picture.width().max(picture.height()) > SIDE {
        picture = picture.resize(SIDE, SIDE, FilterType::CatmullRom);
    }
    encode(&picture)
}

fn encode(picture: &DynamicImage) -> Option<(Vec<u8>, MediaKind)> {
    let mut bytes = Vec::new();
    if is_see_through(picture) {
        let pixels = picture.to_rgba8();
        pixels
            .write_with_encoder(WebPEncoder::new_lossless(&mut bytes))
            .ok()?;
        Some((bytes, MediaKind::Webp))
    } else {
        let pixels = picture.to_rgb8();
        pixels
            .write_with_encoder(JpegEncoder::new_with_quality(&mut bytes, QUALITY))
            .ok()?;
        Some((bytes, MediaKind::Jpeg))
    }
}

fn is_see_through(picture: &DynamicImage) -> bool {
    picture.color().has_alpha() && picture.to_rgba8().pixels().any(|pixel| pixel[3] < 255)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage, Rgba, RgbaImage};

    fn png(picture: DynamicImage) -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        picture.write_to(&mut bytes, ImageFormat::Png).unwrap();
        bytes.into_inner()
    }

    #[test]
    fn makes_a_large_photo_small() {
        let photo = RgbImage::from_fn(3000, 2000, |x, y| Rgb([x as u8, y as u8, (x ^ y) as u8]));
        let (bytes, kind) = prepare(&png(photo.into()), MediaKind::Png).unwrap();
        assert_eq!(kind, MediaKind::Jpeg);
        let small = image::load_from_memory(&bytes).unwrap();
        assert_eq!((small.width(), small.height()), (512, 341));
    }

    #[test]
    fn keeps_a_see_through_background() {
        let sticker = RgbaImage::from_pixel(700, 700, Rgba([9, 9, 9, 40]));
        let (bytes, kind) = prepare(&png(sticker.into()), MediaKind::Png).unwrap();
        assert_eq!(kind, MediaKind::Webp);
        let small = image::load_from_memory(&bytes).unwrap().to_rgba8();
        assert_eq!(small.get_pixel(5, 5)[3], 40);
    }

    #[test]
    fn leaves_an_unreadable_picture_out() {
        assert!(prepare(b"not a picture", MediaKind::Png).is_none());
    }
}
