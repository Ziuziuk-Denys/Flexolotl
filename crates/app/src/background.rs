use crate::{
    platform::tray::{TrayIcon, TrayState},
    shell::Shell,
    window,
};
use gpui_kit::{AnyWindowHandle, App, AppContext, Entity, Global, Subscription};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    Open,
    Toggle,
    Quit,
}

struct Background {
    shell: Entity<Shell>,
    window: Option<AnyWindowHandle>,
    tray: TrayIcon,
    _watch: Subscription,
}

impl Global for Background {}

pub fn start(
    sender: async_channel::Sender<Request>,
    requests: async_channel::Receiver<Request>,
    starts_hidden: bool,
    cx: &mut App,
) {
    let shell = cx.new(Shell::new);
    let tray = TrayIcon::start(tray_state(&shell, cx), sender);
    let watch = cx.observe(&shell, |shell, cx| {
        let state = tray_state(&shell, cx);
        cx.global::<Background>().tray.show(state);
    });
    let has_tray = tray.is_available();
    cx.set_global(Background {
        shell,
        window: None,
        tray,
        _watch: watch,
    });
    cx.on_window_closed(|cx, _| window_closed(cx)).detach();
    cx.spawn(async move |cx| {
        while let Ok(request) = requests.recv().await {
            cx.update(|cx| answer(request, cx));
        }
    })
    .detach();
    if !starts_hidden || !has_tray {
        open(cx);
    }
}

fn answer(request: Request, cx: &mut App) {
    match request {
        Request::Open => open(cx),
        Request::Toggle => {
            let shell = cx.global::<Background>().shell.clone();
            shell.update(cx, |shell, cx| shell.toggle_live(cx));
        }
        Request::Quit => quit(cx),
    }
}

pub fn open(cx: &mut App) {
    let background = cx.global::<Background>();
    let shell = background.shell.clone();
    if let Some(window) = background.window {
        let shown = window.update(cx, |_, window, _| window.activate_window());
        if shown.is_ok() {
            return;
        }
    }
    let window = window::open(shell, cx);
    cx.global_mut::<Background>().window = Some(window);
    cx.activate(true);
}

fn window_closed(cx: &mut App) {
    if !cx.windows().is_empty() {
        return;
    }
    let background = cx.global_mut::<Background>();
    background.window = None;
    let has_tray = background.tray.is_available();
    let stays = background
        .shell
        .clone()
        .read(cx)
        .settings
        .stays_in_background;
    if !(stays && has_tray) {
        quit(cx);
    }
}

#[cfg(target_os = "macos")]
pub fn close(cx: &mut App) {
    if let Some(window) = cx.global::<Background>().window {
        let _ = window.update(cx, |_, window, _| window.remove_window());
    }
}

pub fn quit_later(cx: &mut App) {
    cx.defer(quit);
}

pub fn quit(cx: &mut App) {
    let shell = cx.global::<Background>().shell.clone();
    shell.update(cx, |shell, _| shell.finish());
    cx.quit();
}

fn tray_state(shell: &Entity<Shell>, cx: &App) -> TrayState {
    let shell = shell.read(cx);
    let copy = shell.copy();
    TrayState {
        is_live: shell.is_live,
        can_toggle: shell.is_live || shell.can_show(),
        title: shell.deck.current().title.clone(),
        open: copy.open,
        toggle: if shell.is_live { copy.hide } else { copy.show },
        quit: copy.quit,
    }
}
