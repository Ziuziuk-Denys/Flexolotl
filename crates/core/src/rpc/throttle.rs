use std::time::{Duration, Instant};

const INTERVAL: Duration = Duration::from_secs(4);

#[derive(Debug)]
pub struct Throttle<T> {
    pending: Option<T>,
    last_sent: Option<Instant>,
}

impl<T> Default for Throttle<T> {
    fn default() -> Self {
        Self {
            pending: None,
            last_sent: None,
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
        self.last_sent = Some(now);
        self.pending.take()
    }

    pub fn wait(&self, now: Instant) -> Option<Duration> {
        self.pending.as_ref()?;
        let elapsed = self.last_sent.map_or(INTERVAL, |sent| now - sent);
        Some(INTERVAL.saturating_sub(elapsed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sends_the_first_value_at_once() {
        let mut throttle = Throttle::default();
        throttle.push("a");
        assert_eq!(throttle.take_due(Instant::now()), Some("a"));
    }

    #[test]
    fn keeps_only_the_newest_value_while_waiting() {
        let start = Instant::now();
        let mut throttle = Throttle::default();
        throttle.push(1);
        throttle.take_due(start);
        throttle.push(2);
        throttle.push(3);
        assert_eq!(throttle.take_due(start + Duration::from_secs(1)), None);
        assert_eq!(throttle.take_due(start + INTERVAL), Some(3));
        assert_eq!(throttle.wait(start + INTERVAL), None);
    }
}
