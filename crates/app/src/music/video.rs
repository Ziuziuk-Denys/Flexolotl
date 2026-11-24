use serde_json::{Value, json};

pub const SEARCH: &str = "https://www.youtube.com/youtubei/v1/search?prettyPrint=false&fields=contents(twoColumnSearchResultsRenderer(primaryContents(sectionListRenderer(contents(itemSectionRenderer(contents(videoRenderer(videoId,title(runs)))))))))";
const SECTIONS: &str =
    "/contents/twoColumnSearchResultsRenderer/primaryContents/sectionListRenderer/contents";
const ID_LENGTH: usize = 11;

pub fn question(words: &str) -> String {
    let client = json!({"clientName": "WEB", "clientVersion": "2.20250101.00.00"});
    json!({"context": {"client": client}, "query": words}).to_string()
}

pub fn cover(answer: &Value, title: &str) -> Option<String> {
    let id = answer
        .pointer(SECTIONS)?
        .as_array()?
        .iter()
        .filter_map(|section| section.pointer("/itemSectionRenderer/contents")?.as_array())
        .flatten()
        .map(|item| &item["videoRenderer"])
        .find(|video| {
            let found = video.pointer("/title/runs/0/text").and_then(Value::as_str);
            found.is_some_and(|found| found.trim() == title.trim())
        })?["videoId"]
        .as_str()?;
    let is_plain = id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));
    (id.len() == ID_LENGTH && is_plain)
        .then(|| format!("https://i.ytimg.com/vi/{id}/hqdefault.jpg"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn video(id: &str, title: &str) -> Value {
        json!({"videoRenderer": {"videoId": id, "title": {"runs": [{"text": title}]}}})
    }

    #[test]
    fn takes_the_video_with_exactly_this_title() {
        let videos = json!([
            video("RD_Jwka9NaY", "Another"),
            {},
            video("jNQXAC9IVRw", "A zoo")
        ]);
        let sections = json!([{"itemSectionRenderer": {"contents": videos}}, {}]);
        let answer = json!({"contents": {"twoColumnSearchResultsRenderer": {
            "primaryContents": {"sectionListRenderer": {"contents": sections}}}}});
        assert_eq!(
            cover(&answer, " A zoo").as_deref(),
            Some("https://i.ytimg.com/vi/jNQXAC9IVRw/hqdefault.jpg")
        );
        assert_eq!(cover(&answer, "Missing"), None);
        assert_eq!(cover(&json!({}), "A zoo"), None);
    }
}
