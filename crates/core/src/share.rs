use crate::{activity::MemberLine, status::Status};

const PREFIX: &str = "flx1.";
const MAX_CODE_LENGTH: usize = 8 * 1024;
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub fn encode(status: &Status) -> String {
    let json = serde_json::to_vec(status).unwrap_or_default();
    format!("{PREFIX}{}", to_base64(&json))
}

pub fn decode(code: &str) -> Option<Status> {
    let code = code.trim();
    if code.len() > MAX_CODE_LENGTH {
        return None;
    }
    let json = from_base64(code.strip_prefix(PREFIX)?)?;
    let status: Status = serde_json::from_slice(&json).ok()?;
    let is_valid = status.activity(0, MemberLine::Name).issues().is_empty();
    is_valid.then_some(status)
}

fn to_base64(bytes: &[u8]) -> String {
    bytes
        .chunks(3)
        .flat_map(|chunk| {
            let mut padded = [0; 4];
            padded[1..=chunk.len()].copy_from_slice(chunk);
            let group = u32::from_be_bytes(padded);
            (0..=chunk.len()).map(move |index| {
                let shift = 18 - 6 * index;
                char::from(ALPHABET[(group >> shift) as usize & 63])
            })
        })
        .collect()
}

fn from_base64(text: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(text.len() * 3 / 4);
    let mut buffer = 0u32;
    let mut bits = 0;
    for character in text.bytes() {
        let value = ALPHABET.iter().position(|&letter| letter == character)?;
        buffer = buffer << 6 | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::{Button, Link};

    fn status() -> Status {
        Status {
            title: "Пишу Flexolotl".into(),
            subtitle: "last bug".into(),
            buttons: vec![Button {
                label: "Open".into(),
                link: Link::new("https://nuvo.quest"),
            }],
            ..Status::default()
        }
    }

    #[test]
    fn survives_a_round_trip() {
        for length in 0..5 {
            let status = Status {
                subtitle: "ab".repeat(length) + "c",
                ..status()
            };
            let status = Status {
                subtitle: if length == 0 {
                    String::new()
                } else {
                    status.subtitle
                },
                ..status
            };
            assert_eq!(decode(&encode(&status)), Some(status));
        }
    }

    #[test]
    fn rejects_hostile_input() {
        let mut dangerous = status();
        dangerous.buttons[0].link = Link::new("javascript:alert(1)");
        let long = format!("{PREFIX}{}", "A".repeat(MAX_CODE_LENGTH));
        for code in [
            "",
            "flx1.",
            "flx1.!!!",
            "flx2.AAAA",
            &encode(&dangerous),
            &long,
        ] {
            assert_eq!(decode(code), None, "{}", &code[..code.len().min(40)]);
        }
    }
}
