use crate::theme::{Theme, mix};
use gpui_kit::{
    AnimationExt, IntoElement, ParentElement, SpringAnimation, SpringConfig, Styled, div, px,
};

const SPRING: SpringConfig = SpringConfig::new(520., 38., 1.);

pub fn toggle(id: &'static str, is_on: bool, theme: Theme) -> impl IntoElement {
    let spring = SpringAnimation::new(SPRING).to(f32::from(is_on));
    div()
        .flex_none()
        .w(px(40.))
        .h(px(24.))
        .with_spring(id, spring, move |track, share: f32| {
            track
                .relative()
                .rounded_full()
                .bg(mix(theme.hairline, theme.ink, share.clamp(0., 1.)))
                .child(
                    div()
                        .absolute()
                        .top(px(2.))
                        .left(px(2. + 16. * share))
                        .size(px(20.))
                        .rounded_full()
                        .bg(theme.paper),
                )
        })
}
