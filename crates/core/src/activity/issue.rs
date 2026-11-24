use super::{Activity, MAX_BUTTONS, text};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Name,
    Details,
    DetailsLink,
    State,
    StateLink,
    LargePicture,
    LargeCaption,
    LargeLink,
    SmallPicture,
    SmallCaption,
    SmallLink,
    Party,
    Progress,
    Buttons,
    ButtonLabel(usize),
    ButtonLink(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Problem {
    Missing,
    TooShort { limit: usize },
    TooLong { limit: usize },
    TooMany { limit: usize },
    NotAPublicHttpsLink,
    OutOfOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Issue {
    pub field: Field,
    pub problem: Problem,
}

impl Activity {
    pub fn issues(&self) -> Vec<Issue> {
        let mut checks = vec![
            (Field::Name, text::problem(&self.name)),
            (Field::Details, text::problem(&self.details)),
            (Field::DetailsLink, self.details_link.problem()),
            (Field::State, text::problem(&self.state)),
            (Field::StateLink, self.state_link.problem()),
            (Field::LargePicture, self.large_picture.source_problem()),
            (
                Field::LargeCaption,
                text::problem(&self.large_picture.caption),
            ),
            (Field::LargeLink, self.large_picture.link.problem()),
            (Field::SmallPicture, self.small_picture.source_problem()),
            (
                Field::SmallCaption,
                text::problem(&self.small_picture.caption),
            ),
            (Field::SmallLink, self.small_picture.link.problem()),
            (Field::Party, self.party_problem()),
            (
                Field::Progress,
                self.progress.is_backwards().then_some(Problem::OutOfOrder),
            ),
            (
                Field::Buttons,
                (self.buttons.len() > MAX_BUTTONS)
                    .then_some(Problem::TooMany { limit: MAX_BUTTONS }),
            ),
        ];
        for (index, button) in self.buttons.iter().enumerate() {
            checks.push((Field::ButtonLabel(index), button.label_problem()));
            checks.push((Field::ButtonLink(index), button.link_problem()));
        }
        checks
            .into_iter()
            .filter_map(|(field, problem)| problem.map(|problem| Issue { field, problem }))
            .collect()
    }

    fn party_problem(&self) -> Option<Problem> {
        let [current, max] = self.party?;
        (current == 0 || current > max).then_some(Problem::OutOfOrder)
    }
}
