//! The opening input device selects MenuFlyout's source padding states.

use reveal_embedder::PointerDeviceKind;
use reveal_foundation::{App, Handle};
use reveal_gestures::{GestureBinding, PointerEvent, PointerRoute};
use reveal_widgets::{FocusManager, KeyEventResult};
use std::rc::Rc;

/// WinUI's input categories needed by menu layout on the desktop host.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum InputDevice {
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
    /// Starts tracking before an opening control can receive input.
    pub(super) fn initialize(app: &mut App) {
        let tracker = app.singleton::<InputDeviceTracker>();
        if app.get(tracker).installed {
            return;
        }
        app.get_mut(tracker).installed = true;
        let route = PointerRoute::new(move |app, event| {
            if matches!(event, PointerEvent::Added(_) | PointerEvent::Removed(_)) {
                return;
            }
            app.get_mut(tracker).last = match event.kind() {
                PointerDeviceKind::Touch => Self::Touch,
                PointerDeviceKind::Stylus | PointerDeviceKind::InvertedStylus => Self::Pen,
                PointerDeviceKind::Mouse | PointerDeviceKind::Trackpad => Self::Mouse,
                PointerDeviceKind::Unknown => return,
            };
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

    /// Reads the input category when a menu is opened.
    pub(super) fn current(app: &mut App) -> Self {
        let tracker: Handle<InputDeviceTracker> = app.singleton();
        app.get(tracker).last
    }

    /// MenuFlyoutItemBase::GetShouldBeNarrow.
    pub(super) fn is_narrow(self) -> bool {
        matches!(self, Self::Mouse | Self::Pen | Self::Keyboard)
    }
}
