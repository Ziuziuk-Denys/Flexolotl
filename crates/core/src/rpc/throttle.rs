use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const WINDOW: Duration = Duration::from_secs(20);
const MOST_IN_WINDOW: usize = 5;

#[derive(Debug)]
pub struct Throttle<T> {
    pending: Option<T>,
    sent: VecDeque<Instant>,
}

impl<T> Default for Throttle<T> {
    fn default() -> Self {
        Self {
            pending: None,
            sent: VecDeque::with_capacity(MOST_IN_WINDOW),
        }
    }
}

impl<T> Throttle<T> {
    pub fn push(&mut self, value: T) {
        self.pending = Some(value);
    }

    pub fn clear(&mut self) {
        self.pending = None;
    }

    pub fn take_due(&mut self, now: Instant) -> Option<T> {
        if !self.wait(now)?.is_zero() {
            return None;
        }
        if self.sent.len() == MOST_IN_WINDOW {
            self.sent.pop_front();
        }
        self.sent.push_back(now);
        self.pending.take()
    }

    pub fn wait(&self, now: Instant) -> Option<Duration> {
        self.pending.as_ref()?;
        if self.sent.len() < MOST_IN_WINDOW {
            return Some(Duration::ZERO);
        }
        Some(WINDOW.saturating_sub(now - self.sent[0]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sends_a_burst_at_once_and_then_waits_for_the_window() {
        let start = Instant::now();
        let mut throttle = Throttle::default();
        for value in 0..MOST_IN_WINDOW {
            throttle.push(value);
            assert_eq!(throttle.take_due(start), Some(value));
        }
        throttle.push(9);
        assert_eq!(throttle.take_due(start + Duration::from_secs(19)), None);
        assert_eq!(throttle.take_due(start + WINDOW), Some(9));
        assert_eq!(throttle.wait(start + WINDOW), None);
    }

    #[test]
    fn keeps_only_the_newest_value_while_waiting() {
        let start = Instant::now();
        let mut throttle = Throttle::default();
        for value in 0..=MOST_IN_WINDOW {
            throttle.push(value);
            throttle.take_due(start);
        }
        throttle.push(50);
        throttle.push(60);
        assert_eq!(throttle.take_due(start + WINDOW), Some(60));
    }

    #[test]
    fn lets_a_line_through_every_four_seconds() {
        let start = Instant::now();
        let mut throttle = Throttle::default();
        for step in 0..30 {
            throttle.push(step);
            let now = start + Duration::from_secs(4 * step);
            assert_eq!(throttle.take_due(now), Some(step));
        }
    }
}
