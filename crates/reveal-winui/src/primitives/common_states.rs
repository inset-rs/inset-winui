//! XAML's `CommonStates` visual state group (`Normal`, `PointerOver`, `Pressed`, `Disabled`) as the framework machinery that computes it from the pointer, plus keyboard focus for the system focus visual.

use reveal_foundation::{App, Handle, Listener};
use reveal_rendering::HitTestBehavior;
use reveal_services::{KeyEvent, LogicalKeyboardKey};
use reveal_widgets::*;
use std::{any::TypeId, collections::HashMap, fmt, rc::Rc};

/// The `CommonStates` group every control template declares.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CommonState {
    #[default]
    Normal,
    PointerOver,
    Pressed,
    Disabled,
}

/// What the state tracker knows: the common state and whether the keyboard focus visual shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlStates {
    pub common: CommonState,
    pub focused: bool,
}

/// Builds the control's template for the current states.
pub type StatesBuilder = Rc<dyn Fn(&mut App, BuildContext, ControlStates) -> WidgetRef>;

/// Tracks pointer over, press, keyboard activation and focus for a control and runs `click` on activation; the `ButtonBase` of XAML.
#[derive(Clone)]
pub struct CommonStates {
    pub click: Listener,
    pub builder: StatesBuilder,
    pub is_enabled: bool,
    /// `ButtonBase::SetAcceptsReturn`: whether Enter activates as Space does. `ButtonBase::Initialize` sets it; `CheckBox` and `RadioButton` clear it.
    pub accepts_return: bool,
    pub key: Option<KeyRef>,
}

impl CommonStates {
    pub fn new(
        click: Listener,
        builder: impl Fn(&mut App, BuildContext, ControlStates) -> WidgetRef + 'static,
    ) -> CommonStates {
        CommonStates {
            click,
            builder: Rc::new(builder),
            is_enabled: true,
            accepts_return: true,
            key: None,
        }
    }

    pub fn accepts_return(mut self, accepts_return: bool) -> CommonStates {
        self.accepts_return = accepts_return;
        self
    }

    pub fn is_enabled(mut self, enabled: bool) -> CommonStates {
        self.is_enabled = enabled;
        self
    }

    pub fn key(mut self, key: KeyRef) -> CommonStates {
        self.key = Some(key);
        self
    }
}

impl fmt::Debug for CommonStates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommonStates")
            .field("is_enabled", &self.is_enabled)
            .field("accepts_return", &self.accepts_return)
            .finish_non_exhaustive()
    }
}

pub struct CommonStatesData {
    state: StateData<CommonStates>,
    pressed: bool,
    hovered: bool,
    focused: bool,
    focus_node: Option<AnyFocusNode>,
    pointer_down: bool,
    space_or_enter_key_down: bool,
    gamepad_a_key_down: bool,
    actions: HashMap<TypeId, AnyAction>,
}

impl StatefulWidget for CommonStates {
    type State = CommonStatesData;
    fn key(&self) -> Option<&KeyRef> {
        self.key.as_ref()
    }
    fn create_state(&self) -> CommonStatesData {
        CommonStatesData {
            state: StateData::new(),
            pressed: false,
            hovered: false,
            focused: false,
            focus_node: None,
            pointer_down: false,
            space_or_enter_key_down: false,
            gamepad_a_key_down: false,
            actions: HashMap::new(),
        }
    }
}

impl CommonStatesData {
    fn clear_state_flags(self: Handle<Self>, app: &mut App) {
        self.set_state(app, |state| {
            state.pressed = false;
            state.pointer_down = false;
            state.space_or_enter_key_down = false;
            state.gamepad_a_key_down = false;
        });
    }

    /// `KeyPress::ButtonBase`: a release-mode button presses on key-down and clicks on key-up.
    fn on_key_event(self: Handle<Self>, app: &mut App, event: &KeyEvent) -> KeyEventResult {
        if !self.widget(app).is_enabled {
            return KeyEventResult::Ignored;
        }
        let key = event.logical_key();
        let is_space_or_enter = key == LogicalKeyboardKey::SPACE
            || (self.widget(app).accepts_return
                && matches!(
                    key,
                    LogicalKeyboardKey::ENTER | LogicalKeyboardKey::NUMPAD_ENTER
                ));
        let is_press = is_space_or_enter || key == LogicalKeyboardKey::GAME_BUTTON_A;
        match event {
            KeyEvent::Down(_) | KeyEvent::Repeat(_) => {
                let state = app.get(self);
                if is_press {
                    if !state.pointer_down
                        && !state.space_or_enter_key_down
                        && !state.gamepad_a_key_down
                    {
                        self.set_state(app, |state| {
                            state.space_or_enter_key_down = is_space_or_enter;
                            state.gamepad_a_key_down = !is_space_or_enter;
                            state.pressed = true;
                        });
                    }
                } else if state.space_or_enter_key_down || state.gamepad_a_key_down {
                    self.set_state(app, |state| {
                        state.pressed = false;
                        state.space_or_enter_key_down = false;
                        state.gamepad_a_key_down = false;
                    });
                }
            }
            KeyEvent::Up(_) if is_press => {
                if is_space_or_enter {
                    app.get_mut(self).space_or_enter_key_down = false;
                } else {
                    app.get_mut(self).gamepad_a_key_down = false;
                }
                if !app.get(self).pointer_down {
                    if app.get(self).pressed {
                        self.activate(app);
                    }
                    self.set_state(app, |state| state.pressed = false);
                }
            }
            KeyEvent::Up(_) => {}
        }
        // The app's activation shortcut must not turn an ignored repeat into another click.
        if is_press {
            KeyEventResult::Handled
        } else if matches!(
            key,
            LogicalKeyboardKey::ENTER | LogicalKeyboardKey::NUMPAD_ENTER
        ) {
            KeyEventResult::SkipRemainingHandlers
        } else {
            KeyEventResult::Ignored
        }
    }

    fn activate(self: Handle<Self>, app: &mut App) {
        let widget = self.widget(app).clone();
        if widget.is_enabled {
            widget.click.call(app);
        }
    }

    fn states(self: Handle<Self>, app: &App) -> ControlStates {
        let widget = self.widget(app);
        let data = app.get(self);
        if !widget.is_enabled {
            return ControlStates {
                common: CommonState::Disabled,
                focused: false,
            };
        }
        ControlStates {
            common: if data.pressed {
                CommonState::Pressed
            } else if data.hovered {
                CommonState::PointerOver
            } else {
                CommonState::Normal
            },
            focused: data.focused,
        }
    }
}

impl State for CommonStatesData {
    type Widget = CommonStates;
    reveal_widgets::state_accessors!();

    fn init_state(self: Handle<Self>, app: &mut App) {
        let node = FocusNode::new(app).as_node();
        node.set_on_key_event(
            app,
            Some(Rc::new(move |app, _, event| self.on_key_event(app, event))),
        );
        app.get_mut(self).focus_node = Some(node);
        let action = CallbackAction::<ActivateIntent>::new(
            app,
            Rc::new(move |app, _| {
                self.activate(app);
                None
            }),
        );
        app.get_mut(self)
            .actions
            .insert(TypeId::of::<ActivateIntent>(), action.as_action());
    }

    fn dispose(self: Handle<Self>, app: &mut App) {
        if let Some(node) = app.get_mut(self).focus_node.take() {
            node.dispose(app);
            app.destroy(node.id());
        }
        for action in std::mem::take(&mut app.get_mut(self).actions).into_values() {
            app.destroy(action.id());
        }
    }

    fn did_update_widget(self: Handle<Self>, app: &mut App, _old_widget: &CommonStates) {
        if !self.widget(app).is_enabled {
            self.clear_state_flags(app);
            app.get_mut(self).hovered = false;
        }
    }

    fn build(self: Handle<Self>, app: &mut App, context: BuildContext) -> WidgetRef {
        let widget = self.widget(app).clone();
        let states = self.states(app);
        let body = (widget.builder)(app, context, states);
        let mut gesture = GestureDetector::new()
            .behavior(HitTestBehavior::Opaque)
            .child(body);
        if widget.is_enabled {
            gesture = gesture
                .on_tap_down(Rc::new(move |app, _| {
                    if let Some(node) = app.get(self).focus_node {
                        node.request_focus(app, None);
                    }
                    self.set_state(app, |state| {
                        state.pointer_down = true;
                        state.pressed = true;
                    })
                }))
                .on_tap_up(Rc::new(move |app, _| {
                    self.set_state(app, |state| {
                        state.pointer_down = false;
                        if !state.space_or_enter_key_down && !state.gamepad_a_key_down {
                            state.pressed = false;
                        }
                    })
                }))
                .on_tap_cancel(Listener::new(move |app| {
                    self.set_state(app, |state| {
                        state.pointer_down = false;
                        state.pressed = false;
                    })
                }))
                .on_tap(Listener::new(move |app| {
                    let state = app.get(self);
                    if !state.space_or_enter_key_down && !state.gamepad_a_key_down {
                        self.activate(app);
                    }
                }));
        }
        FocusableActionDetector::new(gesture)
            .focus_node(app.get(self).focus_node.unwrap())
            .enabled(widget.is_enabled)
            .on_focus_change(move |app, focused| {
                if !focused {
                    self.clear_state_flags(app);
                }
            })
            .actions(app.get(self).actions.clone())
            .on_show_focus_highlight(move |app, value| {
                self.set_state(app, |state| state.focused = value)
            })
            .on_show_hover_highlight(move |app, value| {
                self.set_state(app, |state| state.hovered = value)
            })
            .into_widget()
    }
}
