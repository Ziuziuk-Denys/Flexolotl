use super::{ICON, ICON_SIDE, TrayState};
use crate::background::Request;
use ksni::{
    Icon, MenuItem, ToolTip, Tray,
    blocking::{Handle, TrayMethods},
    menu::StandardItem,
};
use std::cell::RefCell;

pub struct TrayIcon {
    handle: Option<Handle<Entry>>,
    shown: RefCell<TrayState>,
}

impl TrayIcon {
    pub fn start(state: TrayState, requests: async_channel::Sender<Request>) -> Self {
        let entry = Entry {
            state: state.clone(),
            requests,
        };
        Self {
            handle: entry.spawn().ok(),
            shown: RefCell::new(state),
        }
    }

    pub fn is_available(&self) -> bool {
        self.handle
            .as_ref()
            .is_some_and(|handle| !handle.is_closed())
    }

    pub fn show(&self, state: TrayState) {
        if *self.shown.borrow() == state {
            return;
        }
        self.shown.replace(state.clone());
        if let Some(handle) = &self.handle {
            handle.update(|entry| entry.state = state);
        }
    }
}

struct Entry {
    state: TrayState,
    requests: async_channel::Sender<Request>,
}

impl Entry {
    fn ask(&self, request: Request) {
        let _ = self.requests.try_send(request);
    }

    fn item(label: &str, request: Request, enabled: bool) -> MenuItem<Self> {
        StandardItem {
            label: label.to_owned(),
            enabled,
            activate: Box::new(move |entry: &mut Self| entry.ask(request)),
            ..StandardItem::default()
        }
        .into()
    }
}

impl Tray for Entry {
    fn id(&self) -> String {
        "flexolotl".to_owned()
    }

    fn title(&self) -> String {
        "Flexolotl".to_owned()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: "Flexolotl".to_owned(),
            description: self.state.description().to_owned(),
            ..ToolTip::default()
        }
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        vec![Icon {
            width: ICON_SIDE as i32,
            height: ICON_SIDE as i32,
            data: ICON.to_vec(),
        }]
    }

    fn activate(&mut self, _: i32, _: i32) {
        self.ask(Request::Open);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            Self::item(self.state.open, Request::Open, true),
            Self::item(self.state.toggle, Request::Toggle, self.state.can_toggle),
            MenuItem::Separator,
            Self::item(self.state.quit, Request::Quit, true),
        ]
    }
}
