use serde_json::Value;

const VIDEO: &str = r#""videoRenderer":{"videoId":""#;
const TITLE: &str = r#""title":{"runs":[{"text":"#;
const ID_LENGTH: usize = 11;

pub fn cover(page: &str, title: &str) -> Option<String> {
    let id = page.split(VIDEO).skip(1).find_map(|video| {
        let found = read_text(video.split_once(TITLE)?.1)?;
        (found.trim() == title.trim()).then(|| video.get(..ID_LENGTH))?
    })?;
    Some(format!("https://i.ytimg.com/vi/{id}/hqdefault.jpg"))
}

fn read_text(json: &str) -> Option<String> {
    let mut values = serde_json::Deserializer::from_str(json).into_iter::<Value>();
    Some(values.next()?.ok()?.as_str()?.to_owned())
}

#[cfg(test)]
mod tests {
    use super::cover;

    const PAGE: &str = r#"{"videoRenderer":{"videoId":"RD_Jwka9NaY","title":{"runs":[{"text":"Another"}]}}},
        {"videoRenderer":{"videoId":"jNQXAC9IVRw","title":{"runs":[{"text":"A \"quoted\" zoo"}]}}}"#;

    #[test]
    fn takes_the_video_with_exactly_this_title() {
        assert_eq!(
            cover(PAGE, r#"A "quoted" zoo"#).as_deref(),
            Some("https://i.ytimg.com/vi/jNQXAC9IVRw/hqdefault.jpg")
        );
        assert_eq!(cover(PAGE, "Missing"), None);
    }
}
