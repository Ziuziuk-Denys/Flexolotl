#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version([u32; 3]);

impl Version {
    pub fn current() -> Self {
        let text = option_env!("FLEXOLOTL_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
        Self::parse(text).unwrap_or(Self([0; 3]))
    }

    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split('.').map(|part| part.parse().ok());
        let version = [parts.next()??, parts.next()??, parts.next()??];
        parts.next().is_none().then_some(Self(version))
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [major, minor, patch] = self.0;
        write!(formatter, "{major}.{minor}.{patch}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_numbers_not_letters() {
        assert!(Version::parse("0.10.0") > Version::parse("0.9.9"));
        for bad in ["", "1.2", "1.2.3.4", "1.2.x", "v1.2.3"] {
            assert_eq!(Version::parse(bad), None, "{bad}");
        }
    }
}
