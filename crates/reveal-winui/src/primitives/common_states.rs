//! XAML's `CommonStates` visual state group (`Normal`, `PointerOver`, `Pressed`, `Disabled`) as the framework machinery that computes it from the pointer, plus keyboard focus for the system focus visual.

use reveal_foundation::{App, Handle, Listener};
use reveal_rendering::HitTestBehavior;
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
            key: None,
        }
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
            .finish_non_exhaustive()
    }
}

pub struct CommonStatesData {
    state: StateData<CommonStates>,
    pressed: bool,
    hovered: bool,
    focused: bool,
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
            actions: HashMap::new(),
        }
    }
}

impl CommonStatesData {
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
        for action in std::mem::take(&mut app.get_mut(self).actions).into_values() {
            app.destroy(action.id());
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
                    self.set_state(app, |state| state.pressed = true)
                }))
                .on_tap_up(Rc::new(move |app, _| {
                    self.set_state(app, |state| state.pressed = false)
                }))
                .on_tap_cancel(Listener::new(move |app| {
                    self.set_state(app, |state| state.pressed = false)
                }))
                .on_tap(Listener::new(move |app| self.activate(app)));
        }
        FocusableActionDetector::new(gesture)
            .enabled(widget.is_enabled)
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
