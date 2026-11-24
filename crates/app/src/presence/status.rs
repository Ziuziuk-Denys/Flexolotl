use flexolotl_core::rpc::Account;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Status {
    pub link: LinkState,
    pub account: Option<Account>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LinkState {
    #[default]
    Connecting,
    DiscordClosed,
    Ready,
    Shown,
    Refused(String),
}
