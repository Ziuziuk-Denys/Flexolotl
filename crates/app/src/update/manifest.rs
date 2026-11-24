use super::Version;
use aws_lc_rs::signature::{ED25519, UnparsedPublicKey};

const RELEASE_KEY: &[u8; 32] = include_bytes!("release-key.bin");
const PREFIX: &str = "Flexolotl-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub version: Version,
    pub name: String,
    pub checksum: [u8; 32],
}

pub fn find_package(sums: &[u8], signature: &[u8], suffix: &str) -> Option<Package> {
    find_signed_package(sums, signature, suffix, RELEASE_KEY)
}

fn find_signed_package(sums: &[u8], signature: &[u8], suffix: &str, key: &[u8]) -> Option<Package> {
    UnparsedPublicKey::new(&ED25519, key)
        .verify(sums, signature)
        .ok()?;
    let text = std::str::from_utf8(sums).ok()?;
    text.lines().find_map(|line| {
        let (checksum, name) = line.split_once("  ")?;
        let version = name.strip_prefix(PREFIX)?.strip_suffix(suffix)?;
        Some(Package {
            version: Version::parse(version.strip_suffix('-')?)?,
            name: name.to_owned(),
            checksum: read_checksum(checksum)?,
        })
    })
}

fn read_checksum(text: &str) -> Option<[u8; 32]> {
    let mut checksum = [0; 32];
    if text.len() != 64 || !text.is_ascii() {
        return None;
    }
    for (byte, digits) in checksum.iter_mut().zip(text.as_bytes().chunks(2)) {
        let digits = std::str::from_utf8(digits).ok()?;
        *byte = u8::from_str_radix(digits, 16).ok()?;
    }
    Some(checksum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_lc_rs::signature::{Ed25519KeyPair, KeyPair};

    fn sums() -> String {
        let checksum = "ab".repeat(32);
        format!(
            "{checksum}  Flexolotl-0.2.0-linux-amd64.deb\n{checksum}  Flexolotl-0.2.0-linux-x86_64.AppImage\n"
        )
    }

    #[test]
    fn trusts_a_real_release() {
        let sums = include_bytes!("fixtures/SHA256SUMS");
        let signature = include_bytes!("fixtures/SHA256SUMS.sig");
        for suffix in ["linux-arm64.tar.gz", "mac-arm64.zip", "win-x64.zip"] {
            let package = find_package(sums, signature, suffix).unwrap();
            assert_eq!(package.version, Version::parse("0.1.2").unwrap());
        }
    }

    #[test]
    fn finds_the_package_for_this_system() {
        let keys = Ed25519KeyPair::generate().unwrap();
        let (sums, key) = (sums(), keys.public_key().as_ref());
        let signature = keys.sign(sums.as_bytes());
        let find = |suffix| find_signed_package(sums.as_bytes(), signature.as_ref(), suffix, key);
        let package = find("linux-x86_64.AppImage").unwrap();
        assert_eq!(package.version, Version::parse("0.2.0").unwrap());
        assert_eq!(package.checksum, [0xab; 32]);
        assert_eq!(find("win-x64.zip"), None);
    }

    #[test]
    fn refuses_what_was_not_signed_by_the_key() {
        let (keys, stranger) = (
            Ed25519KeyPair::generate().unwrap(),
            Ed25519KeyPair::generate().unwrap(),
        );
        let sums = sums();
        let signature = stranger.sign(sums.as_bytes());
        let key = keys.public_key().as_ref();
        let found = |sums: &str, signature: &[u8]| {
            find_signed_package(sums.as_bytes(), signature, "linux-x86_64.AppImage", key)
        };
        assert_eq!(found(&sums, signature.as_ref()), None);
        let changed = sums.replace("0.2.0", "9.2.0");
        assert_eq!(found(&changed, keys.sign(sums.as_bytes()).as_ref()), None);
        assert!(
            find_package(sums.as_bytes(), signature.as_ref(), "linux-x86_64.AppImage").is_none()
        );
    }
}
