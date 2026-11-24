use super::{Status, Timing};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "StoredDeck")]
pub struct Deck {
    statuses: Vec<Status>,
    current: usize,
}

#[derive(Deserialize)]
#[serde(default)]
#[derive(Default)]
struct StoredDeck {
    statuses: Vec<Status>,
    current: usize,
}

impl From<StoredDeck> for Deck {
    fn from(stored: StoredDeck) -> Self {
        let mut deck = Self {
            statuses: stored.statuses,
            current: stored.current,
        };
        deck.repair();
        deck
    }
}

impl Deck {
    pub fn starting_with(title: &str) -> Self {
        Self {
            statuses: vec![Status {
                title: title.to_owned(),
                timing: Timing::Elapsed,
                ..Status::default()
            }],
            current: 0,
        }
    }

    pub fn statuses(&self) -> &[Status] {
        &self.statuses
    }

    pub fn current_index(&self) -> usize {
        self.current
    }

    pub fn current(&self) -> &Status {
        &self.statuses[self.current]
    }

    pub fn current_mut(&mut self) -> &mut Status {
        &mut self.statuses[self.current]
    }

    pub fn select(&mut self, index: usize) {
        self.current = index.min(self.statuses.len() - 1);
    }

    pub fn add(&mut self, status: Status) {
        self.statuses.push(status);
        self.current = self.statuses.len() - 1;
    }

    pub fn remove(&mut self, index: usize) {
        if self.statuses.len() < 2 || index >= self.statuses.len() {
            return;
        }
        self.statuses.remove(index);
        if index < self.current {
            self.current -= 1;
        }
        self.repair();
    }

    fn repair(&mut self) {
        if self.statuses.is_empty() {
            self.statuses.push(Status::default());
        }
        self.current = self.current.min(self.statuses.len() - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deck(titles: &[&str]) -> Deck {
        let mut deck = Deck::starting_with(titles[0]);
        for title in &titles[1..] {
            deck.add(Status {
                title: (*title).to_owned(),
                ..Status::default()
            });
        }
        deck
    }

    #[test]
    fn keeps_the_current_status_when_an_earlier_one_goes() {
        let mut deck = deck(&["a", "b", "c"]);
        deck.select(2);
        deck.remove(0);
        assert_eq!(deck.current().title, "c");
    }

    #[test]
    fn never_becomes_empty() {
        let mut deck = deck(&["a"]);
        deck.remove(0);
        assert_eq!(deck.statuses().len(), 1);
    }

    #[test]
    fn repairs_a_damaged_file() {
        let empty: Deck = serde_json::from_str("{}").unwrap();
        assert_eq!(empty.statuses().len(), 1);
        let beyond: Deck =
            serde_json::from_str(r#"{"statuses":[{"title":"a"}],"current":9}"#).unwrap();
        assert_eq!(beyond.current().title, "a");
    }
}
