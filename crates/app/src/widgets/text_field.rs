use gpui_kit::{
    App, AppContext, Context, Entity, Focusable, Hsla, IntoElement, SharedString, Subscription,
    Window,
    base::input::{Input, InputEditorStyle, InputEvent, InputState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldEvent {
    Changed,
    Submitted,
    Left,
}

#[derive(Clone, Copy)]
pub struct FieldColors {
    pub ink: Hsla,
    pub soft: Hsla,
    pub selection: Hsla,
}

pub struct TextField {
    state: Entity<InputState>,
    _events: Subscription,
}

impl TextField {
    pub fn new<V: 'static>(
        placeholder: impl Into<SharedString>,
        value: impl Into<SharedString>,
        colors: FieldColors,
        on_event: impl Fn(&mut V, FieldEvent, &mut Window, &mut Context<V>) + 'static,
        window: &mut Window,
        cx: &mut Context<V>,
    ) -> Self {
        let state = cx.new(|cx| {
            let mut state = InputState::new(window, cx)
                .placeholder(placeholder)
                .default_value(value);
            state.set_editor_style(InputEditorStyle {
                foreground: colors.ink,
                muted_foreground: colors.soft,
                caret: colors.ink,
                selection: colors.selection,
                ..InputEditorStyle::default()
            });
            state
        });
        let events = cx.subscribe_in(&state, window, move |view, _, event, window, cx| {
            let event = match event {
                InputEvent::Change => FieldEvent::Changed,
                InputEvent::PressEnter { .. } => FieldEvent::Submitted,
                InputEvent::Blur => FieldEvent::Left,
                InputEvent::Focus => return cx.notify(),
            };
            on_event(view, event, window, cx);
        });
        Self {
            state,
            _events: events,
        }
    }

    pub fn element(&self) -> impl IntoElement {
        Input::new(&self.state)
    }

    pub fn text(&self, cx: &App) -> String {
        self.state.read(cx).value().to_string()
    }

    pub fn set_text(&self, text: &str, window: &mut Window, cx: &mut App) {
        let text = text.to_owned();
        self.state
            .update(cx, |state, cx| state.set_value(text, window, cx));
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        self.state.update(cx, |state, cx| state.focus(window, cx));
    }

    pub fn is_focused(&self, window: &Window, cx: &App) -> bool {
        self.state.read(cx).focus_handle(cx).is_focused(window)
    }
}
