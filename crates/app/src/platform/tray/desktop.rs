use super::{ICON, ICON_SIDE, TrayState};
use crate::background::Request;
use std::cell::RefCell;
use tray_icon::{
    Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

pub struct TrayIcon {
    entry: Option<Entry>,
    shown: RefCell<TrayState>,
}

struct Entry {
    icon: tray_icon::TrayIcon,
    open: MenuItem,
    toggle: MenuItem,
    quit: MenuItem,
}

impl TrayIcon {
    pub fn start(state: TrayState, requests: async_channel::Sender<Request>) -> Self {
        Self {
            entry: Entry::new(&state, requests),
            shown: RefCell::new(state),
        }
    }

    pub fn is_available(&self) -> bool {
        self.entry.is_some()
    }

    pub fn show(&self, state: TrayState) {
        if *self.shown.borrow() == state {
            return;
        }
        if let Some(entry) = &self.entry {
            entry.apply(&state);
        }
        self.shown.replace(state);
    }
}

impl Entry {
    fn new(state: &TrayState, requests: async_channel::Sender<Request>) -> Option<Self> {
        let open = MenuItem::new(state.open, true, None);
        let toggle = MenuItem::new(state.toggle, state.can_toggle, None);
        let quit = MenuItem::new(state.quit, true, None);
        let menu = Menu::new();
        menu.append_items(&[&open, &toggle, &PredefinedMenuItem::separator(), &quit])
            .ok()?;
        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(cfg!(target_os = "macos"))
            .with_tooltip("Flexolotl")
            .with_icon(picture()?)
            .build()
            .ok()?;
        let answers = [
            (open.id().clone(), Request::Open),
            (toggle.id().clone(), Request::Toggle),
            (quit.id().clone(), Request::Quit),
        ];
        let clicks = requests.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if let Some((_, request)) = answers.iter().find(|(id, _)| *id == event.id) {
                let _ = requests.try_send(*request);
            }
        }));
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            else {
                return;
            };
            if cfg!(target_os = "windows") {
                let _ = clicks.try_send(Request::Open);
            }
        }));
        Some(Self {
            icon,
            open,
            toggle,
            quit,
        })
    }

    fn apply(&self, state: &TrayState) {
        self.open.set_text(state.open);
        self.toggle.set_text(state.toggle);
        self.toggle.set_enabled(state.can_toggle);
        self.quit.set_text(state.quit);
        let tooltip = match state.description() {
            "" => "Flexolotl".to_owned(),
            title => format!("Flexolotl: {title}"),
        };
        let _ = self.icon.set_tooltip(Some(tooltip));
    }
}

fn picture() -> Option<Icon> {
    let pixels = ICON
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[alpha, red, green, blue]| [red, green, blue, alpha])
        .collect();
    Icon::from_rgba(pixels, ICON_SIDE, ICON_SIDE).ok()
}
