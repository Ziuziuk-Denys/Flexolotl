const STEP: f32 = 1. / 240.;
const STIFFNESS: f32 = 230.;
const DAMPING: f32 = 29.;

#[derive(Debug, Clone, Copy)]
pub struct Spring {
    pub value: f32,
    pub velocity: f32,
    pub target: f32,
    tolerance: f32,
}

impl Spring {
    pub fn resting_at(value: f32, tolerance: f32) -> Self {
        Self {
            value,
            velocity: 0.,
            target: value,
            tolerance,
        }
    }

    pub fn snap_to(&mut self, value: f32) {
        self.value = value;
        self.target = value;
        self.velocity = 0.;
    }

    pub fn is_moving(&self) -> bool {
        self.value != self.target || self.velocity != 0.
    }

    pub fn advance(&mut self, seconds: f32) {
        let mut remaining = seconds;
        while remaining > 0. && self.is_moving() {
            let step = remaining.min(STEP);
            let pull = (self.target - self.value) * STIFFNESS - self.velocity * DAMPING;
            self.velocity += pull * step;
            self.value += self.velocity * step;
            remaining -= step;
            self.settle();
        }
    }

    fn settle(&mut self) {
        let is_close = (self.target - self.value).abs() < self.tolerance;
        let is_slow = self.velocity.abs() < self.tolerance * 10.;
        if is_close && is_slow {
            self.snap_to(self.target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reaches_its_target_and_stops() {
        let mut spring = Spring::resting_at(0., 0.5);
        spring.target = 326.;
        for _ in 0..120 {
            spring.advance(1. / 60.);
        }
        assert_eq!(spring.value, 326.);
        assert!(!spring.is_moving());
    }

    #[test]
    fn keeps_the_speed_it_was_thrown_with() {
        let mut spring = Spring::resting_at(0., 0.5);
        spring.velocity = 900.;
        spring.advance(1. / 60.);
        assert!(spring.value > 8.);
    }
}
