use super::{CARD_WIDTH, palette};
use crate::shell::Shell;
use gpui_kit::{
    AnyElement, BoxShadow, Context, FontWeight, IntoElement, ParentElement, Styled, Window, div,
    img, point, prelude::FluentBuilder, px, rgba,
};

const FONT: &str = "Noto Sans";
const RADIUS: f32 = 8.;
const BANNER_HEIGHT: f32 = 105.;

impl Shell {
    pub(super) fn render_card(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let activity = self.render_activity(window, cx);
        let account = self.connection.account.as_ref();
        let display_name = account.map_or("You", |account| &account.display_name);
        let username = account.map_or("you", |account| &account.username);
        div()
            .w(px(CARD_WIDTH))
            .rounded(px(RADIUS))
            .bg(palette::popout())
            .font_family(FONT)
            .text_color(palette::text())
            .shadow(vec![BoxShadow {
                color: rgba(0x00000073).into(),
                offset: point(px(0.), px(24.)),
                blur_radius: px(44.),
                spread_radius: px(-16.),
                inset: false,
            }])
            .child(
                div()
                    .h(px(BANNER_HEIGHT))
                    .rounded_t(px(RADIUS))
                    .bg(self.portrait.tone),
            )
            .child(
                div()
                    .px(px(16.))
                    .pb(px(16.))
                    .flex()
                    .flex_col()
                    .child(self.render_avatar())
                    .child(
                        div()
                            .mt(px(4.))
                            .truncate()
                            .text_size(px(20.))
                            .line_height(px(24.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(display_name.to_owned()),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(14.))
                            .line_height(px(18.))
                            .text_color(palette::text_soft())
                            .child(username.to_owned()),
                    )
                    .child(activity),
            )
            .into_any_element()
    }

    fn render_avatar(&self) -> impl IntoElement {
        div()
            .relative()
            .mt(px(-46.))
            .ml(px(-6.))
            .size(px(92.))
            .p(px(6.))
            .rounded_full()
            .bg(palette::popout())
            .child(
                div()
                    .size(px(80.))
                    .rounded_full()
                    .bg(palette::artwork())
                    .when_some(self.avatar_url(), |frame, url| {
                        frame.child(img(url).size(px(80.)).rounded_full())
                    }),
            )
            .when_some(self.decoration_url(), |frame, url| {
                frame.child(img(url).absolute().left(px(-2.)).top(px(-2.)).size(px(96.)))
            })
            .child(
                div()
                    .absolute()
                    .right(px(2.))
                    .bottom(px(2.))
                    .size(px(26.))
                    .p(px(5.))
                    .rounded_full()
                    .bg(palette::popout())
                    .child(div().size_full().rounded_full().bg(palette::positive())),
            )
    }
}
