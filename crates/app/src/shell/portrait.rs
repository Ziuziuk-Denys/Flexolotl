use super::Shell;
use crate::art::Portrait;
use gpui_kit::{Context, ImgResourceLoader, Resource, Window};

impl Shell {
    pub(super) fn refresh_portrait(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(url) = self.avatar_url() else {
            return;
        };
        if self.portrait.is_from(&url) {
            return;
        }
        let resource = Resource::Uri(url.clone().into());
        if let Some(Ok(avatar)) = window.use_asset::<ImgResourceLoader>(&resource, cx)
            && let Some(portrait) = Portrait::from_avatar(&url, &avatar)
        {
            self.portrait = portrait;
        }
    }

    pub fn decoration_url(&self) -> Option<String> {
        self.connection.account.as_ref()?.decoration.clone()
    }

    pub fn avatar_url(&self) -> Option<String> {
        self.connection.account.as_ref()?.avatar.clone()
    }
}
