//! XAML's `CInputManager` last-input-device record: menus read it to pick their padding states, and `CFocusManager::CoerceFocusState` reads it to decide whether programmatic focus shows the system focus visual.

use reveal_embedder::PointerDeviceKind;
use reveal_foundation::{App, Handle};
use reveal_gestures::{GestureBinding, PointerEvent, PointerRoute};
use reveal_widgets::{FocusManager, KeyEventResult};
use std::rc::Rc;

/// WinUI's input categories needed by menu layout on the desktop host.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum InputDevice {
    /// No user input has been observed yet.
    #[default]
    None,

    /// A mouse or trackpad pointer.
    Mouse,

    /// Direct touch input uses the larger padding.
    Touch,

    /// Stylus input uses narrow padding.
    Pen,

    /// Keyboard activation uses narrow padding.
    Keyboard,
}

/// Per-App input history; listeners observe the same native routes used by FocusManager.
#[derive(Default)]
struct InputDeviceTracker {
    /// Device associated with the latest observed input.
    last: InputDevice,

    /// Prevents registering the observers more than once.
    installed: bool,
}

impl InputDevice {
    /// Starts tracking before an opening control can receive input; `FocusState::coerce_programmatic` reads `None` until then.
    pub(crate) fn initialize(app: &mut App) {
        let tracker = app.singleton::<InputDeviceTracker>();
        if app.get(tracker).installed {
            return;
        }
        app.get_mut(tracker).installed = true;
        let route = PointerRoute::new(move |app, event| {
            if matches!(event, PointerEvent::Added(_) | PointerEvent::Removed(_)) {
                return;
            }
            Self::pointer(app, event.kind());
        });
        GestureBinding::instance(app)
            .pointer_router(app)
            .add_global_route(app, route, None);
        FocusManager::instance(app).add_early_key_event_handler(
            app,
            Rc::new(move |app, _| {
                app.get_mut(tracker).last = Self::Keyboard;
                KeyEventResult::Ignored
            }),
        );
    }

    /// Raw pointer handlers can open a menu before PointerRouter runs its global observers.
    pub(crate) fn pointer(app: &mut App, kind: PointerDeviceKind) {
        let input = match kind {
            PointerDeviceKind::Touch => Self::Touch,
            PointerDeviceKind::Stylus | PointerDeviceKind::InvertedStylus => Self::Pen,
            PointerDeviceKind::Mouse | PointerDeviceKind::Trackpad => Self::Mouse,
            PointerDeviceKind::Unknown => return,
        };
        let tracker = app.singleton::<InputDeviceTracker>();
        app.get_mut(tracker).last = input;
    }

    /// Reads the input category when a menu is opened.
    pub(crate) fn current(app: &mut App) -> Self {
        let tracker: Handle<InputDeviceTracker> = app.singleton();
        app.get(tracker).last
    }

    /// MenuFlyoutItemBase::GetShouldBeNarrow.
    pub(crate) fn is_narrow(self) -> bool {
        matches!(self, Self::Mouse | Self::Pen | Self::Keyboard)
    }
}

/// XAML `FocusState` as `CFocusManager::CoerceFocusState` records it for the focused element: the cause of the focus change, which decides whether the system focus visual is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum FocusState {
    /// Pointer input, or programmatic focus while the last input was not a keyboard; no focus visual.
    #[default]
    Pointer,

    /// Keyboard input, or programmatic focus while the last input was a keyboard; the focus visual shows.
    Keyboard,
}

impl FocusState {
    /// `CFocusManager::CoerceFocusState` for `FocusState::Programmatic`: the last input device decides, and only a keyboard yields `Keyboard`.
    pub(crate) fn coerce_programmatic(app: &mut App) -> Self {
        match InputDevice::current(app) {
            InputDevice::Keyboard => Self::Keyboard,
            InputDevice::None | InputDevice::Mouse | InputDevice::Touch | InputDevice::Pen => {
                Self::Pointer
            }
        }
    }

    /// Whether `CFocusManager` draws the system focus visual for this state (`GetRealFocusStateForFocusedElement() == FocusState::Keyboard`).
    pub(crate) fn shows_focus_visual(self) -> bool {
        self == Self::Keyboard
    }
}
