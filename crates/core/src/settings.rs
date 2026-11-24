use crate::activity::MemberLine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub language: Language,
    pub appearance: Appearance,
    pub lines: Lines,
    pub motion: Motion,
    pub member_line: MemberLine,
    pub show_on_start: bool,
    pub application_id: String,
    pub stays_in_background: bool,
    pub music_online: bool,
    pub music_in_browsers: bool,
    pub updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::System,
            appearance: Appearance::Auto,
            lines: Lines::Medium,
            motion: Motion::Full,
            member_line: MemberLine::Name,
            show_on_start: false,
            application_id: String::new(),
            stays_in_background: true,
            music_online: true,
            music_in_browsers: true,
            updates: true,
        }
    }
}

impl Settings {
    pub fn application_id(&self) -> &str {
        let id = self.application_id.trim();
        let is_snowflake = (17..=20).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_digit());
        if is_snowflake {
            id
        } else {
            crate::APPLICATION_ID
        }
    }
}

macro_rules! choice {
    ($name:ident { $first:ident $(, $rest:ident)* }) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            #[default]
            $first,
            $($rest),*
        }

        impl $name {
            pub const ALL: &[Self] = &[Self::$first $(, Self::$rest)*];
        }
    };
}

choice!(Language {
    System,
    English,
    Russian
});
choice!(Appearance { Auto, Light, Dark });
choice!(Lines { Medium, Fine, Bold });
choice!(Motion { Full, Reduced });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_an_application_id_that_is_not_a_snowflake() {
        let mut settings = Settings::default();
        for id in ["", "abc", "123", "15748553123834185181999"] {
            settings.application_id = id.to_owned();
            assert_eq!(settings.application_id(), crate::APPLICATION_ID, "{id}");
        }
        settings.application_id = " 1574855312383418999 ".to_owned();
        assert_eq!(settings.application_id(), "1574855312383418999");
    }
}
