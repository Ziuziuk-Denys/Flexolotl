use super::Shell;
use flexolotl_core::settings::Motion;
use gpui_kit::Context;

const TRAVEL: f32 = 28.;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Statuses,
    Settings,
    Plugins,
}

#[derive(Debug, Clone, Copy)]
pub struct Placement {
    pub offset: f32,
    pub opacity: f32,
}

impl Screen {
    fn side(self) -> f32 {
        match self {
            Self::Statuses => -1.,
            Self::Home => 0.,
            Self::Settings => 1.,
            Self::Plugins => 2.,
        }
    }
}

impl Shell {
    pub fn go_home(&mut self, cx: &mut Context<Self>) {
        self.go_to(Screen::Home, cx);
    }

    pub fn go_to(&mut self, screen: Screen, cx: &mut Context<Self>) {
        if screen == self.screen {
            return;
        }
        if screen == Screen::Statuses {
            self.carousel.settle_on(self.deck.current_index());
        }
        self.previous_screen = self.screen;
        self.screen = screen;
        self.is_checking = false;
        self.page.snap_to(0.);
        self.page.target = 1.;
        if self.settings.motion == Motion::Reduced {
            self.page.snap_to(1.);
        }
        cx.notify();
    }

    pub(super) fn settle(&mut self) {
        self.page.snap_to(1.);
        self.reveal.finish();
        self.screen = Screen::Home;
        self.previous_screen = Screen::Home;
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub(super) fn visible_screens(&self) -> Vec<(Screen, Placement)> {
        let arrived = self.page.value;
        let direction = (self.screen.side() - self.previous_screen.side()).signum();
        let mut screens = Vec::with_capacity(2);
        if self.page.is_moving() {
            screens.push((
                self.previous_screen,
                Placement {
                    offset: -direction * TRAVEL * arrived,
                    opacity: (1. - arrived * 1.7).clamp(0., 1.),
                },
            ));
        }
        screens.push((
            self.screen,
            Placement {
                offset: direction * TRAVEL * (1. - arrived),
                opacity: (arrived * 1.6 - 0.12).clamp(0., 1.),
            },
        ));
        screens
    }
}
