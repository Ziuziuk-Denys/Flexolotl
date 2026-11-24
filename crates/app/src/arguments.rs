use std::{env, ffi::OsString, process::Command};

const BACKGROUND: &str = "--background";
const AFTER_UPDATE: &str = "--after-update";
const HEIR: &str = "--heir";
const LIVE_SINCE: &str = "--live-since=";

pub const AUTOSTART: &str = BACKGROUND;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Arguments {
    pub starts_hidden: bool,
    pub is_after_update: bool,
    pub is_heir: bool,
    pub live_since: Option<u64>,
}

impl Arguments {
    pub fn read() -> Self {
        Self::parse(env::args_os().skip(1))
    }

    pub fn continues_earlier_run(self) -> bool {
        self.is_heir || self.is_after_update
    }

    fn parse(arguments: impl Iterator<Item = OsString>) -> Self {
        let mut read = Self::default();
        for argument in arguments {
            match argument.to_str().unwrap_or_default() {
                BACKGROUND => read.starts_hidden = true,
                AFTER_UPDATE => read.is_after_update = true,
                HEIR => read.is_heir = true,
                other => {
                    let since = other.strip_prefix(LIVE_SINCE);
                    read.live_since = read.live_since.or(since.and_then(|ms| ms.parse().ok()));
                }
            }
        }
        read
    }

    pub fn apply(self, command: &mut Command) {
        let flags = [
            (self.starts_hidden, BACKGROUND),
            (self.is_after_update, AFTER_UPDATE),
            (self.is_heir, HEIR),
        ];
        command.args(
            flags
                .iter()
                .filter(|(is_set, _)| *is_set)
                .map(|(_, flag)| flag),
        );
        if let Some(since) = self.live_since {
            command.arg(format!("{LIVE_SINCE}{since}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_back_what_it_passes_on() {
        let arguments = Arguments {
            starts_hidden: true,
            is_after_update: false,
            is_heir: true,
            live_since: Some(1_795_000_000_000),
        };
        let mut command = Command::new("flexolotl");
        arguments.apply(&mut command);
        let passed = command.get_args().map(ToOwned::to_owned);
        assert_eq!(Arguments::parse(passed), arguments);
        assert_eq!(
            Arguments::parse(["--live-since=x".into(), "%U".into()].into_iter()),
            Arguments::default()
        );
    }
}
