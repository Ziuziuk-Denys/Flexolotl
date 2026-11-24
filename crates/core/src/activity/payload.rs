use super::{Activity, Issue, Picture, text};
use serde_json::{Map, Value, json};

impl Activity {
    pub fn payload(&self) -> Result<Value, Vec<Issue>> {
        let issues = self.issues();
        if !issues.is_empty() {
            return Err(issues);
        }
        let mut fields = Map::new();
        fields.insert("type".into(), self.kind.code().into());
        fields.insert("status_display_type".into(), self.member_line.code().into());
        put_text(&mut fields, "name", &self.name);
        put_text(&mut fields, "details", &self.details);
        put_text(&mut fields, "details_url", self.details_link.as_str());
        put_text(&mut fields, "state", &self.state);
        put_text(&mut fields, "state_url", self.state_link.as_str());
        put_object(&mut fields, "assets", self.assets());
        put_object(&mut fields, "timestamps", self.timestamps());
        if let Some(size) = self.party {
            fields.insert("party".into(), json!({ "id": "flexolotl", "size": size }));
        }
        if !self.buttons.is_empty() {
            let buttons: Vec<Value> = self
                .buttons
                .iter()
                .map(|button| json!({ "label": button.label.trim(), "url": button.link.as_str() }))
                .collect();
            fields.insert("buttons".into(), buttons.into());
        }
        Ok(fields.into())
    }

    fn assets(&self) -> Map<String, Value> {
        let mut assets = Map::new();
        put_picture(&mut assets, "large", &self.large_picture);
        put_picture(&mut assets, "small", &self.small_picture);
        assets
    }

    fn timestamps(&self) -> Map<String, Value> {
        let (start, end) = self.progress.bounds();
        [("start", start), ("end", end)]
            .into_iter()
            .filter_map(|(key, time)| Some((key.to_owned(), time?.into())))
            .collect()
    }
}

fn put_text(fields: &mut Map<String, Value>, key: &str, value: &str) {
    if let Some(value) = text::present(value) {
        fields.insert(key.to_owned(), value.into());
    }
}

fn put_object(fields: &mut Map<String, Value>, key: &str, object: Map<String, Value>) {
    if !object.is_empty() {
        fields.insert(key.to_owned(), object.into());
    }
}

fn put_picture(assets: &mut Map<String, Value>, slot: &str, picture: &Picture) {
    if picture.is_empty() {
        return;
    }
    put_text(assets, &format!("{slot}_image"), &picture.source);
    put_text(assets, &format!("{slot}_text"), &picture.caption);
    put_text(assets, &format!("{slot}_url"), picture.link.as_str());
}
