use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub username: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub decoration: Option<String>,
}

impl Account {
    pub(super) fn from_ready(message: &Value) -> Option<Self> {
        let user = &message["data"]["user"];
        let id = user["id"].as_str()?;
        let username = user["username"].as_str()?.to_owned();
        let display_name = user["global_name"].as_str().unwrap_or(&username).to_owned();
        let avatar = user["avatar"]
            .as_str()
            .map(|hash| format!("https://cdn.discordapp.com/avatars/{id}/{hash}.png?size=160"));
        let decoration = user["avatar_decoration_data"]["asset"].as_str().map(|asset| {
            format!(
                "https://cdn.discordapp.com/avatar-decoration-presets/{asset}.png?size=160&passthrough=false"
            )
        });
        Some(Self {
            username,
            display_name,
            avatar,
            decoration,
        })
    }
}
