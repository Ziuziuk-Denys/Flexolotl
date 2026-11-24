use std::time::{Duration, Instant};

const LONGEST_FRAME: Duration = Duration::from_millis(34);

#[derive(Default)]
pub struct FrameClock {
    last: Option<Instant>,
}

impl FrameClock {
    pub fn tick(&mut self, now: Instant) -> f32 {
        let elapsed = self.last.map_or(Duration::ZERO, |last| now - last);
        self.last = Some(now);
        elapsed.min(LONGEST_FRAME).as_secs_f32()
    }

    pub fn rest(&mut self) {
        self.last = None;
    }
}
