#[derive(Debug, Clone, Copy)]
pub struct Tween {
    pub value: f32,
    target: f32,
    duration: f32,
}

impl Tween {
    pub fn resting_at(value: f32, duration: f32) -> Self {
        Self {
            value,
            target: value,
            duration,
        }
    }

    pub fn go_to(&mut self, target: f32) {
        self.target = target;
    }

    pub fn finish(&mut self) {
        self.value = self.target;
    }

    pub fn is_moving(&self) -> bool {
        self.value != self.target
    }

    pub fn advance(&mut self, seconds: f32) {
        let step = seconds / self.duration;
        let distance = self.target - self.value;
        self.value += distance.clamp(-step, step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_exactly_its_duration() {
        let mut tween = Tween::resting_at(0., 0.5);
        tween.go_to(1.);
        tween.advance(0.25);
        assert_eq!(tween.value, 0.5);
        tween.advance(1.);
        assert_eq!(tween.value, 1.);
        assert!(!tween.is_moving());
    }
}
