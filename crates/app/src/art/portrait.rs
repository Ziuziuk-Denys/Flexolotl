use gpui_kit::{RenderImage, Rgba, rgb};
use image::{Frame, RgbaImage};
use std::sync::Arc;

const SIDE: usize = 512;
const BLUR: usize = 2;
const DARK_SHARE: f32 = 0.04;
const BRIGHT_SHARE: f32 = 0.96;

#[derive(Clone)]
pub struct Portrait {
    pub image: Arc<RenderImage>,
    pub tone: Rgba,
    source: Option<String>,
}

impl Portrait {
    pub fn placeholder() -> Self {
        let light = (0..SIDE * SIDE)
            .map(|index| {
                let x = (index % SIDE) as f32 / SIDE as f32 - 0.5;
                let y = (index / SIDE) as f32 / SIDE as f32 - 0.42;
                (1. - (x * x + y * y).sqrt() * 2.2).clamp(0., 1.)
            })
            .collect();
        Self {
            image: into_image(light),
            tone: rgb(0x5d5b55),
            source: None,
        }
    }

    pub fn from_avatar(url: &str, avatar: &RenderImage) -> Option<Self> {
        let size = avatar.size(0);
        let width = usize::try_from(size.width.0).ok()?;
        let height = usize::try_from(size.height.0).ok()?;
        let pixels = avatar.as_bytes(0)?;
        if width == 0 || height == 0 || pixels.len() < width * height * 4 {
            return None;
        }
        let light = blur(&sample_light(pixels, width, height));
        Some(Self {
            image: into_image(stretch(light)),
            tone: average_tone(pixels),
            source: Some(url.to_owned()),
        })
    }

    pub fn is_from(&self, url: &str) -> bool {
        self.source.as_deref() == Some(url)
    }
}

fn sample_light(pixels: &[u8], width: usize, height: usize) -> Vec<f32> {
    (0..SIDE * SIDE)
        .map(|index| {
            let x = (index % SIDE) * width / SIDE;
            let y = (index / SIDE) * height / SIDE;
            let [blue, green, red] = [0, 1, 2].map(|c| f32::from(pixels[(y * width + x) * 4 + c]));
            (red * 0.30 + green * 0.59 + blue * 0.11) / 255.
        })
        .collect()
}

fn blur(light: &[f32]) -> Vec<f32> {
    let rows = blur_along(light, 1, SIDE);
    blur_along(&rows, SIDE, 1)
}

fn blur_along(light: &[f32], step: usize, across: usize) -> Vec<f32> {
    let mut blurred = vec![0.; light.len()];
    for line in 0..SIDE {
        for position in 0..SIDE {
            let first = position.saturating_sub(BLUR);
            let last = (position + BLUR).min(SIDE - 1);
            let sum: f32 = (first..=last)
                .map(|near| light[line * across + near * step])
                .sum();
            blurred[line * across + position * step] = sum / (last - first + 1) as f32;
        }
    }
    blurred
}

fn stretch(light: Vec<f32>) -> Vec<f32> {
    let mut sorted = light.clone();
    sorted.sort_by(f32::total_cmp);
    let at = |share: f32| sorted[((sorted.len() - 1) as f32 * share) as usize];
    let (dark, bright) = (at(DARK_SHARE), at(BRIGHT_SHARE));
    let range = (bright - dark).max(0.08);
    light
        .into_iter()
        .map(|value| crate::theme::smoothstep(0., 1., (value - dark) / range))
        .collect()
}

fn average_tone(pixels: &[u8]) -> Rgba {
    let mut sums = [0u64; 3];
    let mut count = 0u64;
    for pixel in pixels.as_chunks::<4>().0.iter().step_by(16) {
        for (sum, value) in sums.iter_mut().zip(pixel) {
            *sum += u64::from(*value);
        }
        count += 1;
    }
    let [b, g, r] = sums.map(|sum| sum as f32 / count.max(1) as f32 / 255.);
    Rgba { r, g, b, a: 1. }
}

fn into_image(light: Vec<f32>) -> Arc<RenderImage> {
    let pixels = light
        .into_iter()
        .flat_map(|value| {
            let level = (value * 255.) as u8;
            [level, level, level, 255]
        })
        .collect();
    let buffer = RgbaImage::from_raw(SIDE as u32, SIDE as u32, pixels)
        .expect("the buffer matches the portrait size");
    Arc::new(RenderImage::new(vec![Frame::new(buffer)]))
}
