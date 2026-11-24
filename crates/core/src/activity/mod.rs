mod button;
mod issue;
mod kind;
mod link;
mod payload;
mod picture;
mod progress;
mod text;

pub use button::{Button, MAX_BUTTONS};
pub use issue::{Field, Issue, Problem};
pub use kind::{ActivityKind, MemberLine};
pub use link::Link;
pub use picture::Picture;
pub use progress::Progress;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Activity {
    pub name: String,
    pub kind: ActivityKind,
    pub member_line: MemberLine,
    pub details: String,
    pub details_link: Link,
    pub state: String,
    pub state_link: Link,
    pub large_picture: Picture,
    pub small_picture: Picture,
    pub party: Option<[u32; 2]>,
    pub progress: Progress,
    pub buttons: Vec<Button>,
}
