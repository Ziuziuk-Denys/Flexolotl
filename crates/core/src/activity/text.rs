use super::issue::Problem;

pub(super) const MAX_LENGTH: usize = 128;
const MIN_LENGTH: usize = 2;

pub(super) fn problem(text: &str) -> Option<Problem> {
    match text.trim().chars().count() {
        0 => None,
        n if n < MIN_LENGTH => Some(Problem::TooShort { limit: MIN_LENGTH }),
        n if n > MAX_LENGTH => Some(Problem::TooLong { limit: MAX_LENGTH }),
        _ => None,
    }
}

pub(super) fn present(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}
