use crate::window::HEIGHT;
use gpui_kit::{Rgba, rgb, rgba};

pub const FONT: &str = "Golos Text";

const REVEAL_SOURCE: f32 = 0.939;

#[derive(Clone, Copy)]
pub struct Theme {
    pub paper: Rgba,
    pub ink: Rgba,
    pub soft: Rgba,
    pub hairline: Rgba,
}

impl Theme {
    pub fn at_height(reveal: f32, y: f32) -> Self {
        let reach = (REVEAL_SOURCE - y / HEIGHT).abs();
        Self::mixed(smoothstep(reach - 0.16, reach + 0.02, reveal * 1.36 - 0.2))
    }

    pub fn mixed(night: f32) -> Self {
        let day = Self::day();
        let dark = Self::night();
        Self {
            paper: mix(day.paper, dark.paper, night),
            ink: mix(day.ink, dark.ink, night),
            soft: mix(day.soft, dark.soft, night),
            hairline: mix(day.hairline, dark.hairline, night),
        }
    }

    pub fn day() -> Self {
        Self {
            paper: rgb(0xf5f4f0),
            ink: rgb(0x131312),
            soft: rgb(0x68665f),
            hairline: rgba(0x14141221),
        }
    }

    fn night() -> Self {
        Self {
            paper: rgb(0x0b0b0c),
            ink: rgb(0xf0efe9),
            soft: rgb(0xa3a19a),
            hairline: rgba(0xffffff26),
        }
    }
}

pub fn sheet() -> Rgba {
    rgb(0xffffff)
}

pub fn danger() -> Rgba {
    rgb(0xb3261e)
}

pub fn smoothstep(start: f32, end: f32, value: f32) -> f32 {
    let t = ((value - start) / (end - start)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}

pub fn mix(from: Rgba, to: Rgba, share: f32) -> Rgba {
    let blend = |a: f32, b: f32| a + (b - a) * share;
    Rgba {
        r: blend(from.r, to.r),
        g: blend(from.g, to.g),
        b: blend(from.b, to.b),
        a: blend(from.a, to.a),
    }
}
