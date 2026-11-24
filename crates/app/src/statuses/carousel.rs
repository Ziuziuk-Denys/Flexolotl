use crate::{motion::Spring, shell::Shell};
use gpui_kit::Context;
use std::time::{Duration, Instant};

pub const STRIDE: f32 = 326.;
const CLICK_DISTANCE: f32 = 5.;
const THROW_SECONDS: f32 = 0.16;
const FASTEST_THROW: f32 = 3000.;
const EDGE_RESISTANCE: f32 = 0.35;
const STALE_AFTER: Duration = Duration::from_millis(80);
const SCROLL_REST: Duration = Duration::from_millis(170);

pub struct Carousel {
    position: Spring,
    drag: Option<Drag>,
    last_scroll: Option<Instant>,
}

struct Drag {
    start_x: f32,
    start_position: f32,
    last_x: f32,
    last_time: Instant,
    velocity: f32,
    distance: f32,
}

impl Carousel {
    pub fn new(index: usize) -> Self {
        Self {
            position: Spring::resting_at(position_of(index), 0.4),
            drag: None,
            last_scroll: None,
        }
    }

    pub fn left_of(&self, index: usize) -> f32 {
        self.position.value - position_of(index)
    }

    pub fn centred(&self) -> usize {
        (-self.position.target / STRIDE).round().max(0.) as usize
    }

    pub fn is_dragged(&self) -> bool {
        self.drag.is_some()
    }

    pub fn is_moving(&self) -> bool {
        self.drag.is_none() && self.position.is_moving()
    }

    pub fn advance(&mut self, seconds: f32) {
        if self.drag.is_none() {
            self.position.advance(seconds);
        }
    }

    pub fn settle_on(&mut self, index: usize) {
        self.drag = None;
        self.position.snap_to(position_of(index));
    }

    pub fn go_to(&mut self, index: usize) {
        self.position.target = position_of(index);
    }

    pub fn step(&mut self, by: isize, cx: &mut Context<Shell>) {
        let index = self.centred().saturating_add_signed(by);
        self.go_to(index);
        cx.notify();
    }

    pub fn keep_within(&mut self, last: usize) {
        self.position.target = self.position.target.clamp(position_of(last), 0.);
    }

    pub fn grab(&mut self, x: f32) {
        self.drag = Some(Drag {
            start_x: x,
            start_position: self.position.value,
            last_x: x,
            last_time: Instant::now(),
            velocity: 0.,
            distance: 0.,
        });
        self.position.velocity = 0.;
    }

    pub fn drag(&mut self, x: f32, last: usize) -> bool {
        let Some(drag) = &mut self.drag else {
            return false;
        };
        let now = Instant::now();
        let seconds = (now - drag.last_time).as_secs_f32().max(0.001);
        drag.velocity = drag.velocity * 0.6 + (x - drag.last_x) / seconds * 0.4;
        drag.last_x = x;
        drag.last_time = now;
        drag.distance = drag.distance.max((x - drag.start_x).abs());
        let free = drag.start_position + x - drag.start_x;
        self.position.value = resist(free, position_of(last));
        true
    }

    pub fn release(&mut self, last: usize) -> Option<bool> {
        let drag = self.drag.take()?;
        let was_click = drag.distance < CLICK_DISTANCE;
        let is_stale = drag.last_time.elapsed() > STALE_AFTER;
        self.position.velocity = if was_click || is_stale {
            0.
        } else {
            drag.velocity.clamp(-FASTEST_THROW, FASTEST_THROW)
        };
        let thrown = self.position.value + self.position.velocity * THROW_SECONDS;
        self.position.target = landing(drag.start_position, self.position.value, thrown);
        self.keep_within(last);
        Some(was_click)
    }

    pub fn scroll(&mut self, amount: f32, last: usize) -> bool {
        let is_resting = self
            .last_scroll
            .is_none_or(|time| time.elapsed() > SCROLL_REST);
        if amount == 0. || !is_resting {
            return false;
        }
        self.last_scroll = Some(Instant::now());
        self.position.target += STRIDE.copysign(amount);
        self.keep_within(last);
        true
    }

    pub fn index_at(&self, x: f32, width: f32, last: usize) -> Option<usize> {
        (0..=last).find(|&index| {
            let left = self.left_of(index);
            (left..left + width).contains(&x)
        })
    }
}

fn position_of(index: usize) -> f32 {
    -(index as f32) * STRIDE
}

fn landing(start: f32, released: f32, thrown: f32) -> f32 {
    let page = |position: f32| (position / STRIDE).round();
    let nearest = page(released);
    let first = nearest.min(page(start) - 1.);
    let last = nearest.max(page(start) + 1.);
    page(thrown).clamp(first, last) * STRIDE
}

fn resist(position: f32, end: f32) -> f32 {
    if position > 0. {
        position * EDGE_RESISTANCE
    } else if position < end {
        end + (position - end) * EDGE_RESISTANCE
    } else {
        position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_cover_under_the_cursor() {
        let carousel = Carousel::new(1);
        assert_eq!(carousel.index_at(10., 300., 3), Some(1));
        assert_eq!(carousel.index_at(10. + STRIDE, 300., 3), Some(2));
        assert_eq!(carousel.index_at(10. - STRIDE, 300., 3), Some(0));
        assert_eq!(carousel.index_at(310., 300., 3), None);
    }

    #[test]
    fn a_press_without_movement_is_a_click() {
        let mut carousel = Carousel::new(0);
        carousel.grab(100.);
        carousel.drag(102., 3);
        assert_eq!(carousel.release(3), Some(true));
        assert_eq!(carousel.centred(), 0);
    }

    #[test]
    fn a_long_drag_moves_to_the_next_cover() {
        let mut carousel = Carousel::new(0);
        carousel.grab(300.);
        carousel.drag(80., 3);
        assert_eq!(carousel.release(3), Some(false));
        assert_eq!(carousel.centred(), 1);
    }

    #[test]
    fn does_not_leave_its_ends() {
        let mut carousel = Carousel::new(3);
        carousel.scroll(-1., 3);
        assert_eq!(carousel.centred(), 3);
        let mut carousel = Carousel::new(0);
        carousel.scroll(1., 3);
        assert_eq!(carousel.centred(), 0);
    }
}
