//! XAML `RepeatButton` with the `DefaultRepeatButtonStyle` of `RepeatButton_themeresources.xaml`.
//!
//! The template is one `ContentPresenter` (`x:Name="ContentPresenter"`) carrying `Background`, `BorderBrush`, `ButtonBorderThemeThickness`, `CornerRadius`, `ButtonPadding` and `Foreground`, with a `BrushTransition` of 83 ms on `Background`; the `CommonStates` storyboards swap those brushes per state.
//!
//! Behaviour is `RepeatButton_Partial.cpp` over `ButtonBase_Partial.cpp`: `Initialize` sets `ClickMode_Press`, so `ButtonBase::OnPointerPressed` raises `Click` on the left-button press and `KeyPress::ButtonBase::KeyDown` on the Space or Enter key down; `RepeatButton::OnPointerPressed` and `OnKeyDown` then start a `TimelineTimer` at `Delay`, whose `TickCallback` raises `Click` while the button is pressed with the pointer over it (or the key held) and restarts the timer at `Interval`; `OnPointerReleased`, `OnKeyUp`, `OnLostFocus` and `OnIsEnabledChanged` stop it. Those overrides sit around `CommonStates` (the `ButtonBase`) as a pointer `Listener` and a `Focus` key handler, the way the C++ overrides sit over its base.

use crate::{
    BUTTON_BORDER_THEME_THICKNESS, BUTTON_PADDING, BackgroundSizing, Brush,
    CONTROL_CONTENT_FONT_SIZE, CONTROL_CORNER_RADIUS, CONTROL_FASTER_ANIMATION_DURATION,
    ColorTransition, CommonState, CommonStates, ControlBorder, ControlStates, FocusVisual,
    RepeatButtonResources, ThemeResources, control_text_style,
};
use reveal_embedder::{Color, FontWeight, Offset};
use reveal_foundation::{App, Handle, Listener, Timer};
use reveal_gestures::{K_PRIMARY_BUTTON, PointerDownEvent, PointerMoveEvent};
use reveal_services::{KeyEvent, LogicalKeyboardKey};
use reveal_widgets::Listener as PointerListener;
use reveal_widgets::*;
use std::{fmt, rc::Rc, time::Duration};

/// The default `Delay`, 500 ms: `RepeatButton_Delay` in `DependencyProperty.cpp`.
pub const REPEAT_BUTTON_DELAY: Duration = Duration::from_millis(500);
/// The default `Interval`, 33 ms: `RepeatButton_Interval` in `DependencyProperty.cpp`.
pub const REPEAT_BUTTON_INTERVAL: Duration = Duration::from_millis(33);
/// The style's `BorderThickness` is `ButtonBorderThemeThickness`.
pub const REPEAT_BUTTON_BORDER_THICKNESS: f64 = BUTTON_BORDER_THEME_THICKNESS[0];
/// The `FocusVisualMargin` the style sets.
pub const REPEAT_BUTTON_FOCUS_VISUAL_MARGIN: [f64; 4] = [-3.0; 4];

/// The brushes the style resolves to in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RepeatButtonBrushes {
    pub background: Color,
    pub border: Brush,
    pub foreground: Color,
}

/// The `{ThemeResource …}` values the `CommonStates` storyboards name for `state`.
pub fn repeat_button_brushes(
    resources: &RepeatButtonResources,
    state: CommonState,
) -> RepeatButtonBrushes {
    let r = resources;
    match state {
        CommonState::Normal => RepeatButtonBrushes {
            background: r.repeat_button_background,
            border: Brush::ControlElevation(r.repeat_button_border_brush),
            foreground: r.repeat_button_foreground,
        },
        CommonState::PointerOver => RepeatButtonBrushes {
            background: r.repeat_button_background_pointer_over,
            border: Brush::ControlElevation(r.repeat_button_border_brush_pointer_over),
            foreground: r.repeat_button_foreground_pointer_over,
        },
        CommonState::Pressed => RepeatButtonBrushes {
            background: r.repeat_button_background_pressed,
            border: Brush::Solid(r.repeat_button_border_brush_pressed),
            foreground: r.repeat_button_foreground_pressed,
        },
        CommonState::Disabled => RepeatButtonBrushes {
            background: r.repeat_button_background_disabled,
            border: Brush::Solid(r.repeat_button_border_brush_disabled),
            foreground: r.repeat_button_foreground_disabled,
        },
    }
}

/// `KeyPress::ButtonBase::IsPress` with `AcceptsReturn` (every `ButtonBase::Initialize` sets it): Space or Enter.
fn is_press_key(key: LogicalKeyboardKey) -> bool {
    key == LogicalKeyboardKey::SPACE
        || key == LogicalKeyboardKey::ENTER
        || key == LogicalKeyboardKey::NUMPAD_ENTER
}

/// XAML `RepeatButton`: `Click` on the press, then again after `Delay` and every `Interval` while held.
#[derive(Clone)]
pub struct RepeatButton {
    pub content: WidgetRef,
    pub click: Listener,
    /// XAML `Delay`: the time from the press to the first repeat.
    pub delay: Duration,
    /// XAML `Interval`: the time between repeats.
    pub interval: Duration,
    pub is_enabled: bool,
    pub key: Option<KeyRef>,
}

impl RepeatButton {
    pub fn new<K>(content: impl IntoWidget<K>, click: Listener) -> RepeatButton {
        RepeatButton {
            content: content.into_widget(),
            click,
            delay: REPEAT_BUTTON_DELAY,
            interval: REPEAT_BUTTON_INTERVAL,
            is_enabled: true,
            key: None,
        }
    }

    /// A text `Content`, as `<RepeatButton Content="…"/>`.
    pub fn text(text: impl Into<String>, click: Listener) -> RepeatButton {
        RepeatButton::new(Text::new(text), click)
    }

    /// XAML `Delay`.
    pub fn delay(mut self, delay: Duration) -> RepeatButton {
        self.delay = delay;
        self
    }

    /// XAML `Interval`; `OnIntervalPropertyChanged` rejects a value that is not positive.
    pub fn interval(mut self, interval: Duration) -> RepeatButton {
        assert!(
            !interval.is_zero(),
            "RepeatButton.Interval must be positive"
        );
        self.interval = interval;
        self
    }

    /// XAML `IsEnabled`.
    pub fn is_enabled(mut self, enabled: bool) -> RepeatButton {
        self.is_enabled = enabled;
        self
    }

    pub fn key(mut self, key: KeyRef) -> RepeatButton {
        self.key = Some(key);
        self
    }
}

impl fmt::Debug for RepeatButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RepeatButton")
            .field("delay", &self.delay)
            .field("interval", &self.interval)
            .field("is_enabled", &self.is_enabled)
            .finish_non_exhaustive()
    }
}

impl StatefulWidget for RepeatButton {
    type State = RepeatButtonState;
    fn key(&self) -> Option<&KeyRef> {
        self.key.as_ref()
    }
    fn create_state(&self) -> RepeatButtonState {
        RepeatButtonState {
            state: StateData::new(),
            timer: None,
            pointer_causing_repeat: false,
            keyboard_causing_repeat: false,
            pointer_over: false,
        }
    }
}

/// The repeat machinery of `RepeatButton_Partial.cpp`.
pub struct RepeatButtonState {
    state: StateData<RepeatButton>,
    /// `m_tpTimer`, `Some` while the `TimelineTimer` is enabled.
    timer: Option<Timer>,
    /// `m_pointerCausingRepeat`.
    pointer_causing_repeat: bool,
    /// `m_keyboardCausingRepeat`.
    keyboard_causing_repeat: bool,
    /// `IsPointerOver` for the pressed pointer, from its position against the control's bounds.
    pointer_over: bool,
}

impl RepeatButtonState {
    /// `RepeatButton::OnClick` → `ButtonBase::OnClick`: the `Click` event.
    fn on_click(self: Handle<Self>, app: &mut App) {
        self.widget(app).click.clone().call(app);
    }

    /// `RepeatButton::OnPointerPressed` over `ButtonBase::OnPointerPressed`: with the left button, the base raises `Click` (`ClickMode_Press`), then the pointer causes repeats.
    fn on_pointer_pressed(self: Handle<Self>, app: &mut App, event: PointerDownEvent) {
        if !self.widget(app).is_enabled || event.buttons != K_PRIMARY_BUTTON {
            return;
        }
        app.get_mut(self).pointer_over = true;
        self.on_click(app);
        app.get_mut(self).pointer_causing_repeat = true;
        self.update_repeat_state(app);
    }

    /// `OnPointerEntered` / `OnPointerExited` for the captured pointer: re-entering restarts the repeat (`UpdateRepeatState`); leaving lets the next tick stop it.
    fn on_pointer_moved(self: Handle<Self>, app: &mut App, event: PointerMoveEvent) {
        if !app.get(self).pointer_causing_repeat {
            return;
        }
        let over = self.contains(app, event.local_position());
        let was_over = std::mem::replace(&mut app.get_mut(self).pointer_over, over);
        if over && !was_over {
            self.update_repeat_state(app);
        }
    }

    /// `RepeatButton::OnPointerReleased`, and the capture lost: the pointer stops causing repeats.
    fn on_pointer_released(self: Handle<Self>, app: &mut App) {
        app.get_mut(self).pointer_causing_repeat = false;
        self.update_repeat_state(app);
    }

    /// `RepeatButton::OnKeyDown` / `OnKeyUp` over `KeyPress::ButtonBase::KeyDown` / `KeyUp`: Space or Enter down starts the repeat and raises `Click` (`ClickMode_Press`), up stops it; a key already down (`m_bIsSpaceOrEnterKeyDown`) does nothing more.
    fn on_key_event(self: Handle<Self>, app: &mut App, event: &KeyEvent) -> KeyEventResult {
        if !is_press_key(event.logical_key()) || !self.widget(app).is_enabled {
            return KeyEventResult::Ignored;
        }
        match event {
            KeyEvent::Down(_) if !app.get(self).keyboard_causing_repeat => {
                app.get_mut(self).keyboard_causing_repeat = true;
                self.update_repeat_state(app);
                self.on_click(app);
            }
            KeyEvent::Down(_) | KeyEvent::Repeat(_) => {}
            KeyEvent::Up(_) => {
                app.get_mut(self).keyboard_causing_repeat = false;
                self.update_repeat_state(app);
            }
        }
        KeyEventResult::Handled
    }

    /// `RepeatButton::OnLostFocus` and `OnIsEnabledChanged`: neither input causes repeats any more.
    fn reset_repeat(self: Handle<Self>, app: &mut App) {
        let state = app.get_mut(self);
        state.keyboard_causing_repeat = false;
        state.pointer_causing_repeat = false;
        self.update_repeat_state(app);
    }

    /// `RepeatButton::UpdateRepeatState`.
    fn update_repeat_state(self: Handle<Self>, app: &mut App) {
        if self.should_repeat(app) {
            self.start_timer(app);
        } else {
            self.stop_timer(app);
        }
    }

    /// `(m_pointerCausingRepeat && IsPointerOver) || m_keyboardCausingRepeat`, the condition of `UpdateRepeatState` and of `TickCallback`.
    fn should_repeat(self: Handle<Self>, app: &App) -> bool {
        let state = app.get(self);
        (state.pointer_causing_repeat && state.pointer_over) || state.keyboard_causing_repeat
    }

    /// `RepeatButton::StartTimer`: a timer not already running starts at `Delay`.
    fn start_timer(self: Handle<Self>, app: &mut App) {
        if app.get(self).timer.is_some() {
            return;
        }
        let delay = self.widget(app).delay;
        self.schedule(app, delay);
    }

    /// `RepeatButton::StopTimer`.
    fn stop_timer(self: Handle<Self>, app: &mut App) {
        if let Some(timer) = app.get_mut(self).timer.take() {
            timer.cancel(app);
        }
    }

    /// `TimelineTimer::Start` with the interval it currently holds.
    fn schedule(self: Handle<Self>, app: &mut App, duration: Duration) {
        let timer = Timer::new(app, duration, Listener::new(move |app| self.tick(app)));
        app.get_mut(self).timer = Some(timer);
    }

    /// `TimelineTimer::TimerCallback`: `RepeatButton::TickCallback` raises `Click` while the repeat condition holds and the timer restarts at `Interval`; otherwise it stops.
    fn tick(self: Handle<Self>, app: &mut App) {
        app.get_mut(self).timer = None;
        if !self.should_repeat(app) {
            return;
        }
        self.on_click(app);
        if !self.mounted(app) {
            return;
        }
        let interval = self.widget(app).interval;
        self.schedule(app, interval);
    }

    /// `IsPointerOver`: whether `local` lies inside the control's bounds.
    fn contains(self: Handle<Self>, app: &App, local: Offset) -> bool {
        self.context(app)
            .find_render_object(app)
            .and_then(|object| object.as_box())
            .is_some_and(|render_box| render_box.size(app).contains(local))
    }
}

impl State for RepeatButtonState {
    type Widget = RepeatButton;
    reveal_widgets::state_accessors!();

    fn did_update_widget(self: Handle<Self>, app: &mut App, old_widget: &RepeatButton) {
        // `RepeatButton::OnIsEnabledChanged`.
        if self.widget(app).is_enabled != old_widget.is_enabled {
            self.reset_repeat(app);
        }
    }

    fn dispose(self: Handle<Self>, app: &mut App) {
        self.stop_timer(app);
    }

    fn build(self: Handle<Self>, app: &mut App, _context: BuildContext) -> WidgetRef {
        let widget = self.widget(app).clone();
        let content = widget.content.clone();
        // `ButtonBase` with `ClickMode_Press`: its release activation is unused.
        let presenter = CommonStates::new(Listener::new(|_| {}), move |app, context, states| {
            template(app, context, content.clone(), states)
        })
        .is_enabled(widget.is_enabled);
        // `RepeatButton::OnPointerPressed` / `OnPointerMoved` / `OnPointerReleased`.
        let pointer = PointerListener::new()
            .on_pointer_down(Rc::new(move |app, event| {
                self.on_pointer_pressed(app, event)
            }))
            .on_pointer_move(Rc::new(move |app, event| self.on_pointer_moved(app, event)))
            .on_pointer_up(Rc::new(move |app, _| self.on_pointer_released(app)))
            .on_pointer_cancel(Rc::new(move |app, _| self.on_pointer_released(app)))
            .child(presenter);
        // `RepeatButton::OnKeyDown` / `OnKeyUp` / `OnLostFocus`, ahead of the app's `ActivateIntent` shortcuts; not a focus stop of its own.
        Focus::new(pointer)
            .can_request_focus(false)
            .skip_traversal(true)
            .on_key_event(Rc::new(move |app, _, event| self.on_key_event(app, event)))
            .on_focus_change(move |app, has_focus| {
                if !has_focus {
                    self.reset_repeat(app);
                }
            })
            .into_widget()
    }
}

/// The `ControlTemplate` of `DefaultRepeatButtonStyle` for one set of states.
fn template(
    app: &mut App,
    context: BuildContext,
    content: WidgetRef,
    states: ControlStates,
) -> WidgetRef {
    let resources = ThemeResources::of(app, context);
    let brushes = repeat_button_brushes(&resources.repeat_button(), states.common);
    let text = control_text_style(
        CONTROL_CONTENT_FONT_SIZE,
        FontWeight::W400,
        brushes.foreground,
    );
    // `ContentPresenter`: `HorizontalContentAlignment` and `VerticalContentAlignment` are Center.
    let label = Center::new()
        .width_factor(1.0)
        .height_factor(1.0)
        .child(DefaultTextStyle::new(text, content))
        .into_widget();
    let presenter = ColorTransition::new(
        brushes.background,
        CONTROL_FASTER_ANIMATION_DURATION,
        move |_app, background| {
            ControlBorder::new(Brush::Solid(background), brushes.border)
                .border_thickness(REPEAT_BUTTON_BORDER_THICKNESS)
                .corner_radius(CONTROL_CORNER_RADIUS[0])
                .background_sizing(BackgroundSizing::InnerBorderEdge)
                .padding(BUTTON_PADDING)
                .child(label.clone())
                .into_widget()
        },
    );
    FocusVisual::new(presenter, resources.theme)
        .visible(states.focused)
        .margin(REPEAT_BUTTON_FOCUS_VISUAL_MARGIN)
        .corner_radius(CONTROL_CORNER_RADIUS[0])
        .into_widget()
}
