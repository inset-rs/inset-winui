//! XAML `Slider` with the `DefaultSliderStyle` of `Slider_themeresources.xaml`.
//!
//! Template, top to bottom: a root `Grid` (rows Auto, *) holding `HeaderContentPresenter` in row 0 and, in row 1, `FocusBorder` (the focus visual's target) over `SliderContainer`, the grid that takes the pointer. Inside it `HorizontalTemplate` (columns Auto, Auto, *; rows `SliderPreContentMargin`, Auto, `SliderPostContentMargin`) holds `HorizontalTrackRect` (4 pt tall, every column), `HorizontalDecreaseRect` (column 0, as wide as the value's share of the track), `TopTickBar`, `HorizontalInlineTickBar`, `BottomTickBar` and `HorizontalThumb` (18 × 18, column 1, every row); `VerticalTemplate` is the same turned (rows *, Auto, Auto; columns 14, Auto, 14) with `VerticalTrackRect`, `VerticalDecreaseRect` (row 2), `LeftTickBar`, `VerticalInlineTickBar`, `RightTickBar` and `VerticalThumb`. `Slider::UpdateTrackLayout` swaps the star and the first Auto definition and moves the decrease rect to the other end when `IsDirectionReversed`. The thumb's `SliderThumbStyle` is a `Border` two pixels larger than the thumb on every side (`Margin="-2"`, `SliderThumbCornerRadius`, the elevation stroke) around the 12 pt `SliderInnerThumb` ellipse, which the thumb's own `CommonStates` scale.
//!
//! Behaviour is `Slider_Partial.cpp` (with `Thumb_Partial.cpp` for the thumb's drag and `RangeBase.cpp` for the range): a press on `SliderContainer` moves the thumb to the pointer (`OnPointerPressed`, `MoveThumbToPoint`) and keeps following it (`OnPointerMoved`); a press on the thumb drags it by deltas from the value it started at (`OnThumbDragStarted`, `OnThumbDragDelta`); both set `IntermediateValue`, which positions the thumb, and snap `Value` to the nearest `StepFrequency` multiple (`GetClosestStep`); the arrow keys step by `SmallChange`, Home and End go to the ends (`SliderKeyProcess.h`).

use crate::{
    Brush, CONTROL_CONTENT_FONT_SIZE, CONTROL_CORNER_RADIUS, CONTROL_FAST_ANIMATION_DURATION,
    CONTROL_NORMAL_ANIMATION_DURATION, ColumnDefinition, CommonState, ControlBorder, FocusVisual,
    Grid, GridCell, GridLength, RangeBase, RowDefinition, SLIDER_HEADER_THEME_FONT_WEIGHT,
    SLIDER_HORIZONTAL_HEIGHT, SLIDER_HORIZONTAL_THUMB_HEIGHT, SLIDER_HORIZONTAL_THUMB_WIDTH,
    SLIDER_INNER_THUMB_HEIGHT, SLIDER_INNER_THUMB_WIDTH, SLIDER_OUTSIDE_TICK_BAR_THEME_HEIGHT,
    SLIDER_POST_CONTENT_MARGIN, SLIDER_PRE_CONTENT_MARGIN, SLIDER_THUMB_CORNER_RADIUS,
    SLIDER_TOP_HEADER_MARGIN, SLIDER_TRACK_CORNER_RADIUS, SLIDER_TRACK_THEME_HEIGHT,
    SLIDER_VERTICAL_THUMB_HEIGHT, SLIDER_VERTICAL_THUMB_WIDTH, SLIDER_VERTICAL_WIDTH,
    SliderResources, ThemeResources, TickBar, TickPlacement, are_close, control_text_style,
    fast_out_slow_in, fractional,
};
use reveal_embedder::{Canvas, Color, Offset, PointerDeviceKind, Rect, Size};
use reveal_foundation::{App, Handle};
use reveal_gestures::{EagerGestureRecognizer, K_PRIMARY_BUTTON, PointerDownEvent};
use reveal_painting::{AlignmentGeometry, EdgeInsetsGeometry, draw_oval};
use reveal_rendering::{BoxConstraints, CustomPainter, HitTestBehavior};
use reveal_services::{KeyEvent, LogicalKeyboardKey};
use reveal_widgets::Listener as PointerListener;
use reveal_widgets::*;
use std::{any::TypeId, fmt, rc::Rc, time::Duration};

/// XAML `Orientation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

/// XAML `SliderSnapsTo`: what `Value` snaps to while the thumb moves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SliderSnapsTo {
    /// Multiples of `StepFrequency`.
    #[default]
    StepValues,
    /// Multiples of `TickFrequency`.
    Ticks,
}

/// `Slider_Partial.h`: `Slider::GetDefaultValue2` raises `RangeBase`'s `Maximum` to 100.
pub const SLIDER_DEFAULT_MAXIMUM: f64 = 100.0;
/// `Slider_Partial.h`: `SmallChange`, the arrow keys' step, defaults to 1.
pub const SLIDER_DEFAULT_SMALL_CHANGE: f64 = 1.0;
/// `StepFrequency`'s default of 1 is property metadata the repository does not carry (`PORTING.md`).
pub const SLIDER_DEFAULT_STEP_FREQUENCY: f64 = 1.0;
/// `FocusVisualMargin` of the style, around `FocusBorder`.
pub const SLIDER_FOCUS_VISUAL_MARGIN: [f64; 4] = [-7.0, 0.0, -7.0, 0.0];
/// `SliderThumbStyle`: `BorderThickness="1"`.
pub const THUMB_BORDER_THICKNESS: f64 = 1.0;
/// `SliderThumbStyle`: the `Margin="-2"` its `Border` grows past the thumb by, on every side.
pub const THUMB_BORDER_OVERHANG: f64 = 2.0;
/// The tick bars' `Margin` away from the track: `0,0,0,4` on `TopTickBar`, `0,4,0,0` on `BottomTickBar`, and the same turned on `LeftTickBar` and `RightTickBar`.
pub const TICK_BAR_MARGIN: f64 = 4.0;

/// `SliderInnerThumb`'s `CompositeTransform` scale per thumb state, and the key time it reaches it with `ControlFastOutSlowInKeySpline`: 0.86 over `ControlFastAnimationDuration` (`Normal`), 1.167 over `ControlNormalAnimationDuration` (`PointerOver`), 0.71 over `ControlNormalAnimationDuration` (`Pressed`), 1.167 over `ControlFastAnimationDuration` (`Disabled`).
pub fn inner_thumb_scale(state: CommonState) -> (f64, Duration) {
    match state {
        CommonState::Normal => (0.86, CONTROL_FAST_ANIMATION_DURATION),
        CommonState::PointerOver => (1.167, CONTROL_NORMAL_ANIMATION_DURATION),
        CommonState::Pressed => (0.71, CONTROL_NORMAL_ANIMATION_DURATION),
        CommonState::Disabled => (1.167, CONTROL_FAST_ANIMATION_DURATION),
    }
}

/// XAML `ValueChanged` handler, given the value the slider wants.
pub type ValueChangedHandler = Rc<dyn Fn(&mut App, f64)>;

/// XAML `Slider`: `Value` between `Minimum` and `Maximum` with `ValueChanged`, `StepFrequency`, ticks, `Orientation` and `Header`.
#[derive(Clone)]
pub struct Slider {
    pub value: f64,
    /// XAML `ValueChanged`, with the value the slider wants; the caller owns `value`.
    pub value_changed: ValueChangedHandler,
    pub minimum: f64,
    pub maximum: f64,
    /// XAML `StepFrequency`: `Value` snaps to its multiples.
    pub step_frequency: f64,
    /// XAML `SmallChange`: the arrow keys' step.
    pub small_change: f64,
    /// XAML `TickFrequency`: the tick bars' interval; nothing shows while it is 0.
    pub tick_frequency: f64,
    pub tick_placement: TickPlacement,
    pub snaps_to: SliderSnapsTo,
    pub orientation: Orientation,
    /// XAML `IsDirectionReversed`: the value grows toward the start of the track.
    pub is_direction_reversed: bool,
    pub header: Option<WidgetRef>,
    pub is_enabled: bool,
    pub key: Option<KeyRef>,
}

impl Slider {
    pub fn new(value: f64, value_changed: impl Fn(&mut App, f64) + 'static) -> Slider {
        Slider {
            value,
            value_changed: Rc::new(value_changed),
            minimum: 0.0,
            maximum: SLIDER_DEFAULT_MAXIMUM,
            step_frequency: SLIDER_DEFAULT_STEP_FREQUENCY,
            small_change: SLIDER_DEFAULT_SMALL_CHANGE,
            tick_frequency: 0.0,
            tick_placement: TickPlacement::None,
            snaps_to: SliderSnapsTo::StepValues,
            orientation: Orientation::Horizontal,
            is_direction_reversed: false,
            header: None,
            is_enabled: true,
            key: None,
        }
    }

    /// XAML `Minimum`.
    pub fn minimum(mut self, minimum: f64) -> Slider {
        self.minimum = minimum;
        self
    }

    /// XAML `Maximum`.
    pub fn maximum(mut self, maximum: f64) -> Slider {
        self.maximum = maximum;
        self
    }

    /// XAML `StepFrequency`.
    pub fn step_frequency(mut self, frequency: f64) -> Slider {
        self.step_frequency = frequency;
        self
    }

    /// XAML `SmallChange`.
    pub fn small_change(mut self, change: f64) -> Slider {
        self.small_change = change;
        self
    }

    /// XAML `TickFrequency`.
    pub fn tick_frequency(mut self, frequency: f64) -> Slider {
        self.tick_frequency = frequency;
        self
    }

    /// XAML `TickPlacement`.
    pub fn tick_placement(mut self, placement: TickPlacement) -> Slider {
        self.tick_placement = placement;
        self
    }

    /// XAML `SnapsTo`.
    pub fn snaps_to(mut self, snaps_to: SliderSnapsTo) -> Slider {
        self.snaps_to = snaps_to;
        self
    }

    /// XAML `Orientation`.
    pub fn orientation(mut self, orientation: Orientation) -> Slider {
        self.orientation = orientation;
        self
    }

    /// XAML `IsDirectionReversed`.
    pub fn is_direction_reversed(mut self, reversed: bool) -> Slider {
        self.is_direction_reversed = reversed;
        self
    }

    /// XAML `Header`.
    pub fn header<K>(mut self, header: impl IntoWidget<K>) -> Slider {
        self.header = Some(header.into_widget());
        self
    }

    /// XAML `IsEnabled`.
    pub fn is_enabled(mut self, enabled: bool) -> Slider {
        self.is_enabled = enabled;
        self
    }

    pub fn key(mut self, key: KeyRef) -> Slider {
        self.key = Some(key);
        self
    }

    /// `Minimum`, `Maximum` and `Value` as `CRangeBase` coerces them.
    fn range(&self) -> RangeBase {
        RangeBase::new(self.minimum, self.maximum, self.value).coerced()
    }

    /// The interval `Value` snaps to: `TickFrequency` under `SnapsTo="Ticks"`, `StepFrequency` otherwise.
    fn snap_interval(&self) -> f64 {
        match self.snaps_to {
            SliderSnapsTo::StepValues => self.step_frequency,
            SliderSnapsTo::Ticks => self.tick_frequency,
        }
    }

    /// The thumb's extent along the orientation: `Slider::GetThumbLength`.
    fn thumb_length(&self) -> f64 {
        match self.orientation {
            Orientation::Horizontal => SLIDER_HORIZONTAL_THUMB_WIDTH,
            Orientation::Vertical => SLIDER_VERTICAL_THUMB_HEIGHT,
        }
    }

    /// `Slider::GetClosestStep`: the multiple of `step_delta` nearest `from_value`, inside the range.
    fn closest_step(&self, step_delta: f64, from_value: f64) -> f64 {
        let range = self.range();
        let num_steps = from_value / step_delta;
        let next_step = range.maximum.min(num_steps.ceil() * step_delta);
        let prev_step = range.minimum.max(num_steps.floor() * step_delta);
        if next_step - from_value < from_value - prev_step {
            next_step
        } else {
            prev_step
        }
    }

    /// `Slider::Step` with `bUseSmallChange`: the value one `SmallChange` (or one tick) from `Value` in `forward`'s direction, kept on the step grid; stepping back from the far end lands on the last multiple before it rather than skipping it.
    fn step(&self, forward: bool) -> f64 {
        let step_delta = match self.snaps_to {
            SliderSnapsTo::StepValues => self.small_change,
            SliderSnapsTo::Ticks => self.tick_frequency,
        };
        let range = self.range();
        let value = range.value;
        if !forward
            && are_close(value, range.maximum)
            && !are_close(fractional(value / step_delta), 0.0)
        {
            (value / step_delta).floor() * step_delta
        } else {
            let new_value = if forward {
                value + step_delta
            } else {
                value - step_delta
            };
            self.closest_step(step_delta, new_value)
        }
    }
}

impl fmt::Debug for Slider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Slider")
            .field("value", &self.value)
            .field("minimum", &self.minimum)
            .field("maximum", &self.maximum)
            .field("orientation", &self.orientation)
            .field("is_enabled", &self.is_enabled)
            .finish_non_exhaustive()
    }
}

/// The slider's own state (`Slider_Partial.h` members) and its thumb's (`Thumb_Partial.h`).
pub struct SliderState {
    state: StateData<Slider>,
    focus_node: Option<AnyFocusNode>,
    /// `m_IsPointerOver`: a non-touch pointer is over the control.
    is_pointer_over: bool,
    /// `m_isPressed`: the track is pressed and the thumb follows the pointer.
    is_pressed: bool,
    /// `Thumb::IsDragging`.
    is_dragging: bool,
    /// The thumb's own `m_IsPointerOver`.
    is_thumb_pointer_over: bool,
    /// The keyboard focus visual shows (`Focused`).
    focused: bool,
    /// `m_DragValue`: the value the drag started at plus the deltas since.
    drag_value: f64,
    /// `IntermediateValue` while a press or drag is in progress; `Value` otherwise.
    intermediate_value: Option<f64>,
    /// `Thumb::m_previousPosition`: where the last drag delta was measured from.
    previous_position: Offset,
    /// Names `SliderContainer`, whose `ActualWidth` / `ActualHeight` the pointer maths reads.
    container_key: Rc<GlobalKey>,
}

impl StatefulWidget for Slider {
    type State = SliderState;
    fn key(&self) -> Option<&KeyRef> {
        self.key.as_ref()
    }
    fn create_state(&self) -> SliderState {
        SliderState {
            state: StateData::new(),
            focus_node: None,
            is_pointer_over: false,
            is_pressed: false,
            is_dragging: false,
            is_thumb_pointer_over: false,
            focused: false,
            drag_value: 0.0,
            intermediate_value: None,
            previous_position: Offset::ZERO,
            container_key: Rc::new(GlobalKey::new()),
        }
    }
}

impl SliderState {
    /// `Slider::ChangeVisualState`: the `CommonStates` group of the slider.
    fn slider_state(self: Handle<Self>, app: &App) -> CommonState {
        let data = app.get(self);
        if !self.widget(app).is_enabled {
            CommonState::Disabled
        } else if data.is_pressed {
            CommonState::Pressed
        } else if data.is_pointer_over {
            CommonState::PointerOver
        } else {
            CommonState::Normal
        }
    }

    /// `Thumb::ChangeVisualState`: the `CommonStates` group of the thumb, pressed while it drags.
    fn thumb_state(self: Handle<Self>, app: &App) -> CommonState {
        let data = app.get(self);
        if !self.widget(app).is_enabled {
            CommonState::Disabled
        } else if data.is_dragging {
            CommonState::Pressed
        } else if data.is_thumb_pointer_over {
            CommonState::PointerOver
        } else {
            CommonState::Normal
        }
    }

    /// The value the thumb is laid out at: `IntermediateValue`.
    fn intermediate_value(self: Handle<Self>, app: &App) -> f64 {
        app.get(self)
            .intermediate_value
            .unwrap_or_else(|| self.widget(app).range().value)
    }

    /// `SliderContainer`'s `ActualWidth` and `ActualHeight` (`Slider::UpdateTrackLayout`, `MoveThumbToPoint`), from its render object; zero before it is laid out.
    fn track_size(self: Handle<Self>, app: &mut App) -> Size {
        let key = app.get(self).container_key.clone();
        key.current_context(app)
            .and_then(|context| context.find_render_object(app))
            .and_then(|object| object.as_box())
            .map(|render_box| render_box.size(app))
            .unwrap_or(Size::ZERO)
    }

    /// The thumb's layout slot in `SliderContainer` coordinates, for the value the thumb is at.
    fn thumb_rect(self: Handle<Self>, app: &mut App) -> Rect {
        let track_size = self.track_size(app);
        let widget = self.widget(app);
        let fraction = widget.range().fraction_of(self.intermediate_value(app));
        thumb_rect(
            widget.orientation,
            widget.is_direction_reversed,
            track_size,
            fraction,
        )
    }

    /// Whether `point` lands on the thumb: its `Border` reaches `THUMB_BORDER_OVERHANG` past the slot.
    fn hits_thumb(self: Handle<Self>, app: &mut App, point: Offset) -> bool {
        self.thumb_rect(app)
            .inflate(THUMB_BORDER_OVERHANG)
            .contains(point)
    }

    /// `put_Value`: coerced by `CRangeBase`; `ValueChanged` only when it differs.
    fn put_value(self: Handle<Self>, app: &mut App, value: f64) {
        let widget = self.widget(app).clone();
        let value = widget.range().coerce_value(value);
        if value != widget.range().value {
            (widget.value_changed)(app, value);
        }
    }

    /// The tail of `MoveThumbToPoint` and `OnThumbDragDelta`: `Value` goes to the step nearest `intermediate_value` when that is not where it already is.
    fn snap_value(self: Handle<Self>, app: &mut App, intermediate_value: f64) {
        let widget = self.widget(app).clone();
        let closest_step = widget.closest_step(widget.snap_interval(), intermediate_value);
        if !are_close(widget.range().value, closest_step) {
            self.put_value(app, closest_step);
        }
    }

    /// `Slider::MoveThumbToPoint`: the share of the clickable track (the track less the thumb, whose midpoint never reaches the ends) under `point` becomes `IntermediateValue`, and `Value` snaps to it.
    fn move_thumb_to_point(self: Handle<Self>, app: &mut App, point: Offset) {
        let widget = self.widget(app).clone();
        let track_size = self.track_size(app);
        let (track_length, click_delta) = match widget.orientation {
            Orientation::Horizontal => (track_size.width(), point.dx()),
            Orientation::Vertical => (track_size.height(), track_size.height() - point.dy()),
        };
        let thumb_length = widget.thumb_length();
        let clickable_length = (track_length - thumb_length).max(1.0);
        let mut click_percentage = (click_delta - thumb_length / 2.0) / clickable_length;
        click_percentage = click_percentage.clamp(0.0, 1.0);
        if widget.is_direction_reversed {
            click_percentage = 1.0 - click_percentage;
        }
        let range = widget.range();
        let intermediate_value = range.minimum + click_percentage * range.span();
        self.set_state(app, |state| {
            state.intermediate_value = Some(intermediate_value)
        });
        self.snap_value(app, intermediate_value);
    }

    /// `Slider::OnThumbDragDelta`: the pointer's movement along the track, as a share of the track less the thumb, scaled to the range and added to `m_DragValue`.
    fn thumb_drag_delta(self: Handle<Self>, app: &mut App, change: Offset) {
        let widget = self.widget(app).clone();
        let track_size = self.track_size(app);
        let range = widget.range();
        let (change, actual_size) = match widget.orientation {
            Orientation::Horizontal => (change.dx(), track_size.width()),
            Orientation::Vertical => (-change.dy(), track_size.height()),
        };
        let offset = change * range.span() / (actual_size - widget.thumb_length()).max(1.0);
        if !offset.is_finite() {
            return;
        }
        let drag_value = app.get(self).drag_value
            + if widget.is_direction_reversed {
                -offset
            } else {
                offset
            };
        let intermediate_value = range.coerce_value(drag_value);
        self.set_state(app, |state| {
            state.drag_value = drag_value;
            state.intermediate_value = Some(intermediate_value);
        });
        self.snap_value(app, intermediate_value);
    }

    /// `Slider::OnPointerPressed` for the left button on `SliderContainer`, or `Thumb::OnPointerPressed` when the thumb is under it (the thumb handles the event first).
    fn pointer_pressed(self: Handle<Self>, app: &mut App, event: &PointerDownEvent) {
        if event.buttons & K_PRIMARY_BUTTON == 0 {
            return;
        }
        let point = event.local_position();
        if let Some(node) = app.get(self).focus_node {
            node.request_focus(app, None);
        }
        if self.hits_thumb(app, point) {
            // `OnThumbDragStarted`: the drag accumulates from the current value.
            let value = self.widget(app).range().value;
            self.set_state(app, |state| {
                state.is_dragging = true;
                state.previous_position = point;
                state.drag_value = value;
            });
        } else {
            self.move_thumb_to_point(app, point);
            self.set_state(app, |state| state.is_pressed = true);
        }
    }

    /// `Thumb::OnPointerMoved` while dragging, else `Slider::OnPointerMoved` while pressed.
    fn pointer_moved(self: Handle<Self>, app: &mut App, point: Offset) {
        let data = app.get(self);
        if data.is_dragging {
            let previous = data.previous_position;
            if point != previous {
                app.get_mut(self).previous_position = point;
                self.thumb_drag_delta(app, point - previous);
            }
        } else if data.is_pressed {
            self.move_thumb_to_point(app, point);
        }
    }

    /// `Thumb::OnPointerReleased` (`DragCompleted`) and `Slider::OnPointerReleased` (`PerformPointerUpAction`): `IntermediateValue` settles on `Value`; the thumb's hover state is what the pointer is now over.
    fn pointer_released(self: Handle<Self>, app: &mut App, point: Offset, kind: PointerDeviceKind) {
        let over_thumb = kind != PointerDeviceKind::Touch && self.hits_thumb(app, point);
        self.set_state(app, |state| {
            state.is_dragging = false;
            state.is_pressed = false;
            state.intermediate_value = None;
            state.is_thumb_pointer_over = over_thumb;
        });
    }

    /// `Slider::OnPointerCaptureLost` and `Thumb::OnPointerCaptureLost`: everything the pointer held is let go.
    fn pointer_capture_lost(self: Handle<Self>, app: &mut App) {
        self.set_state(app, |state| {
            state.is_dragging = false;
            state.is_pressed = false;
            state.is_pointer_over = false;
            state.is_thumb_pointer_over = false;
            state.intermediate_value = None;
        });
    }

    /// `Thumb::OnPointerEntered` / `OnPointerExited` for a pointer moving over the control without pressing (touch is ignored there); while the container holds the pointer the thumb sees nothing.
    fn pointer_hovered(self: Handle<Self>, app: &mut App, point: Offset, kind: PointerDeviceKind) {
        let data = app.get(self);
        if kind == PointerDeviceKind::Touch || data.is_pressed || data.is_dragging {
            return;
        }
        let was_over_thumb = data.is_thumb_pointer_over;
        let over_thumb = self.hits_thumb(app, point);
        if over_thumb != was_over_thumb {
            self.set_state(app, |state| state.is_thumb_pointer_over = over_thumb);
        }
    }

    /// `KeyPress::Slider::KeyDown`: the arrow keys step by `SmallChange` (Left and Down toward `Minimum`, mirrored by `IsDirectionReversed`), Home and End go to the ends.
    fn key_down(self: Handle<Self>, app: &mut App, key: LogicalKeyboardKey) -> KeyEventResult {
        let widget = self.widget(app).clone();
        if !widget.is_enabled {
            return KeyEventResult::Ignored;
        }
        let reversed = widget.is_direction_reversed;
        let value = match key {
            LogicalKeyboardKey::ARROW_LEFT => widget.step(reversed),
            LogicalKeyboardKey::ARROW_RIGHT => widget.step(!reversed),
            LogicalKeyboardKey::ARROW_UP => widget.step(!reversed),
            LogicalKeyboardKey::ARROW_DOWN => widget.step(reversed),
            LogicalKeyboardKey::HOME => widget.range().minimum,
            LogicalKeyboardKey::END => widget.range().maximum,
            _ => return KeyEventResult::Ignored,
        };
        self.put_value(app, value);
        KeyEventResult::Handled
    }

    /// The pointer handlers `Slider::OnApplyTemplate` attaches to `SliderContainer`, on the container; the thumb's are folded in by hit-testing the thumb's slot.
    fn pointer_listener(self: Handle<Self>, container: WidgetRef) -> PointerListener {
        PointerListener::new()
            .behavior(HitTestBehavior::Opaque)
            .on_pointer_down(Rc::new(move |app: &mut App, event| {
                self.pointer_pressed(app, &event);
            }))
            .on_pointer_move(Rc::new(move |app: &mut App, event| {
                self.pointer_moved(app, event.local_position());
            }))
            .on_pointer_up(Rc::new(move |app: &mut App, event| {
                self.pointer_released(app, event.local_position(), event.kind);
            }))
            .on_pointer_cancel(Rc::new(move |app: &mut App, _event| {
                self.pointer_capture_lost(app);
            }))
            .on_pointer_hover(Rc::new(move |app: &mut App, event| {
                self.pointer_hovered(app, event.local_position(), event.kind);
            }))
            .child(container)
    }
}

impl State for SliderState {
    type Widget = Slider;
    reveal_widgets::state_accessors!();

    fn init_state(self: Handle<Self>, app: &mut App) {
        let node = FocusNode::new(app).as_node();
        node.set_on_key_event(
            app,
            Some(Rc::new(
                move |app: &mut App, _node, event: &KeyEvent| match event {
                    KeyEvent::Down(down) => self.key_down(app, down.logical_key),
                    KeyEvent::Repeat(repeat) => self.key_down(app, repeat.logical_key),
                    KeyEvent::Up(_) => KeyEventResult::Ignored,
                },
            )),
        );
        app.get_mut(self).focus_node = Some(node);
    }

    fn did_update_widget(self: Handle<Self>, app: &mut App, _old_widget: &Slider) {
        // `Slider::OnIsEnabledChanged`: a disabled slider is neither hovered nor pressed.
        if !self.widget(app).is_enabled {
            let state = app.get_mut(self);
            state.is_pointer_over = false;
            state.is_pressed = false;
            state.is_dragging = false;
            state.is_thumb_pointer_over = false;
            state.intermediate_value = None;
        }
    }

    fn dispose(self: Handle<Self>, app: &mut App) {
        if let Some(node) = app.get_mut(self).focus_node.take() {
            node.dispose(app);
        }
    }

    fn build(self: Handle<Self>, app: &mut App, context: BuildContext) -> WidgetRef {
        let widget = self.widget(app).clone();
        let resources = ThemeResources::of(app, context);
        let states = TemplateStates {
            slider: self.slider_state(app),
            thumb: self.thumb_state(app),
            focused: app.get(self).focused && widget.is_enabled,
            intermediate_value: self.intermediate_value(app),
            zoom_scale: MediaQuery::device_pixel_ratio_of(app, context),
        };
        let container_key: KeyRef = app.get(self).container_key.clone();
        let container = slider_container(&resources, &widget, states, container_key);
        let container = if widget.is_enabled {
            // `ManipulationMode="None"`: the slider keeps every pointer that lands on it.
            RawGestureDetector::new()
                .gestures(vec![(
                    TypeId::of::<EagerGestureRecognizer>(),
                    GestureRecognizerFactoryWithHandlers::new(
                        EagerGestureRecognizer::new,
                        |_, _| {},
                    )
                    .into_factory(),
                )])
                .child(self.pointer_listener(container))
                .into_widget()
        } else {
            container
        };
        let body = template(&resources, &widget, states, container);
        let mut detector = FocusableActionDetector::new(body)
            .enabled(widget.is_enabled)
            .on_show_focus_highlight(move |app, value| {
                self.set_state(app, |state| state.focused = value)
            })
            .on_show_hover_highlight(move |app, value| {
                // `Slider::OnPointerEntered` / `OnPointerExited`; the thumb exits with the control.
                self.set_state(app, |state| {
                    state.is_pointer_over = value;
                    if !value {
                        state.is_thumb_pointer_over = false;
                    }
                })
            });
        if let Some(node) = app.get(self).focus_node {
            detector = detector.focus_node(node);
        }
        detector.into_widget()
    }
}

/// What the template is built for: the two `CommonStates` groups, focus, the thumb's value and the pixel scale.
#[derive(Clone, Copy, Debug, PartialEq)]
struct TemplateStates {
    slider: CommonState,
    thumb: CommonState,
    focused: bool,
    intermediate_value: f64,
    zoom_scale: f64,
}

/// The brushes the slider's `CommonStates` storyboards set for one state.
struct SliderBrushes {
    /// `HorizontalTrackRect` / `VerticalTrackRect` `Fill`: the control's `Background`.
    track: Color,
    /// `HorizontalDecreaseRect` / `VerticalDecreaseRect` `Fill`: the control's `Foreground`.
    decrease: Color,
    /// `HorizontalThumb` / `VerticalThumb` `Background`, which `SliderInnerThumb` fills with.
    thumb: Color,
    /// `SliderContainer` `Background`.
    container: Color,
    /// `HeaderContentPresenter` `Foreground`.
    header: Color,
    /// The outside tick bars' `Fill`.
    tick_bar: Color,
}

fn brushes(resources: &ThemeResources, r: &SliderResources, state: CommonState) -> SliderBrushes {
    match state {
        CommonState::Normal => SliderBrushes {
            track: r.slider_track_fill,
            decrease: r.slider_track_value_fill,
            thumb: r.slider_thumb_background,
            // `SliderContainerBackground`: `ControlFillColorTransparentBrush` in the Light dictionary; the Default dictionary omits the key.
            container: resources.common.control_fill_color_transparent,
            header: r.slider_header_foreground,
            tick_bar: r.slider_tick_bar_fill,
        },
        CommonState::PointerOver => SliderBrushes {
            track: r.slider_track_fill_pointer_over,
            decrease: r.slider_track_value_fill_pointer_over,
            thumb: r.slider_thumb_background_pointer_over,
            container: r.slider_container_background_pointer_over,
            header: r.slider_header_foreground,
            tick_bar: r.slider_tick_bar_fill,
        },
        CommonState::Pressed => SliderBrushes {
            track: r.slider_track_fill_pressed,
            decrease: r.slider_track_value_fill_pressed,
            thumb: r.slider_thumb_background_pressed,
            container: r.slider_container_background_pressed,
            header: r.slider_header_foreground,
            tick_bar: r.slider_tick_bar_fill,
        },
        CommonState::Disabled => SliderBrushes {
            track: r.slider_track_fill_disabled,
            decrease: r.slider_track_value_fill_disabled,
            thumb: r.slider_thumb_background_disabled,
            container: r.slider_container_background_disabled,
            header: r.slider_header_foreground_disabled,
            tick_bar: r.slider_tick_bar_fill_disabled,
        },
    }
}

/// `Slider::UpdateTrackLayout`: the decrease rect's length is `multiplier × (track − thumb)`, never negative.
fn decrease_length(track_length: f64, thumb_length: f64, fraction: f64) -> f64 {
    (fraction * (track_length - thumb_length)).max(0.0)
}

/// The thumb's slot in the template grid: after the decrease rect along the track (before it when reversed), centred across it in the three rows or columns.
fn thumb_rect(
    orientation: Orientation,
    is_direction_reversed: bool,
    track_size: Size,
    fraction: f64,
) -> Rect {
    let across =
        (SLIDER_PRE_CONTENT_MARGIN + SLIDER_TRACK_THEME_HEIGHT + SLIDER_POST_CONTENT_MARGIN
            - SLIDER_HORIZONTAL_THUMB_HEIGHT)
            / 2.0;
    match orientation {
        Orientation::Horizontal => {
            let thumb = SLIDER_HORIZONTAL_THUMB_WIDTH;
            let decrease = decrease_length(track_size.width(), thumb, fraction);
            let left = if is_direction_reversed {
                track_size.width() - decrease - thumb
            } else {
                decrease
            };
            Rect::from_ltwh(left, across, thumb, SLIDER_HORIZONTAL_THUMB_HEIGHT)
        }
        Orientation::Vertical => {
            let thumb = SLIDER_VERTICAL_THUMB_HEIGHT;
            let decrease = decrease_length(track_size.height(), thumb, fraction);
            let top = if is_direction_reversed {
                decrease
            } else {
                track_size.height() - decrease - thumb
            };
            Rect::from_ltwh(across, top, SLIDER_VERTICAL_THUMB_WIDTH, thumb)
        }
    }
}

/// The root `Grid`: `HeaderContentPresenter` in the Auto row, `FocusBorder` and `SliderContainer` in the star row.
fn template(
    resources: &ThemeResources,
    widget: &Slider,
    states: TemplateStates,
    container: WidgetRef,
) -> WidgetRef {
    let brushes = brushes(resources, &resources.slider(), states.slider);
    let mut children = Vec::new();
    if let Some(header) = &widget.header {
        // `HeaderContentPresenter`: `SliderHeaderThemeFontWeight` is Normal; collapsed without a `Header`.
        let [left, top, right, bottom] = SLIDER_TOP_HEADER_MARGIN;
        let text = control_text_style(
            CONTROL_CONTENT_FONT_SIZE,
            SLIDER_HEADER_THEME_FONT_WEIGHT,
            brushes.header,
        );
        children.push(
            GridCell::new(
                Padding::new(EdgeInsetsGeometry::from_ltrb(left, top, right, bottom))
                    .child(DefaultTextStyle::new(text, header.clone())),
            )
            .row(0)
            .into_widget(),
        );
    }
    // `FocusBorder`: the system focus visual around row 1, with the style's `FocusVisualMargin`.
    children.push(
        GridCell::new(
            FocusVisual::new(container, resources.theme)
                .visible(states.focused)
                .margin(SLIDER_FOCUS_VISUAL_MARGIN)
                .corner_radius(CONTROL_CORNER_RADIUS[0]),
        )
        .row(1)
        .into_widget(),
    );
    Grid::new()
        .row_definitions([
            RowDefinition::new(GridLength::AUTO),
            RowDefinition::new(GridLength::STAR),
        ])
        .children(children)
        .into_widget()
}

/// `SliderContainer`: its background and the template for the orientation.
fn slider_container(
    resources: &ThemeResources,
    widget: &Slider,
    states: TemplateStates,
    key: KeyRef,
) -> WidgetRef {
    let brushes = brushes(resources, &resources.slider(), states.slider);
    let template = match widget.orientation {
        Orientation::Horizontal => horizontal_template(resources, widget, states, &brushes),
        Orientation::Vertical => vertical_template(resources, widget, states, &brushes),
    };
    ColoredBox::new(brushes.container)
        .key(key)
        .child(template)
        .into_widget()
}

/// `Slider::UpdateTrackLayout` gives the decrease rect `multiplier × (track − thumb)` of the star space and the rest to the star column or row beyond the thumb; the same split as star weights, which need no actual size at build time.
fn decrease_definitions(fraction: f64, is_direction_reversed: bool) -> (GridLength, GridLength) {
    let decrease = GridLength::star(fraction);
    let remainder = GridLength::star(1.0 - fraction);
    if is_direction_reversed {
        (remainder, decrease)
    } else {
        (decrease, remainder)
    }
}

/// A track `Rectangle`: `RadiusX` / `RadiusY` from the control's `CornerRadius` (`SliderTrackCornerRadius`).
fn track_rectangle(fill: Color) -> ControlBorder {
    ControlBorder::new(
        Brush::Solid(fill),
        Brush::Solid(Color::from_argb(0, 0, 0, 0)),
    )
    .border_thickness(0.0)
    .corner_radius(SLIDER_TRACK_CORNER_RADIUS[0])
}

/// A `TickBar` of the slider with `fill`, laid out from the slider's properties.
fn tick_bar(widget: &Slider, states: TemplateStates, fill: Color) -> TickBar {
    let range = widget.range();
    TickBar::new(fill, widget.orientation)
        .tick_frequency(widget.tick_frequency)
        .range(range.minimum, range.maximum)
        .thumb_length(widget.thumb_length())
        .is_direction_reversed(widget.is_direction_reversed)
        .zoom_scale(states.zoom_scale)
}

/// `HorizontalTemplate`.
fn horizontal_template(
    resources: &ThemeResources,
    widget: &Slider,
    states: TemplateStates,
    brushes: &SliderBrushes,
) -> WidgetRef {
    let reversed = widget.is_direction_reversed;
    let fraction = widget.range().fraction_of(states.intermediate_value);
    let placement = widget.tick_placement;
    let mut children = vec![
        // `HorizontalTrackRect`.
        GridCell::new(
            SizedBox::new()
                .height(SLIDER_TRACK_THEME_HEIGHT)
                .child(track_rectangle(brushes.track)),
        )
        .row(1)
        .column_span(3)
        .into_widget(),
        // `HorizontalDecreaseRect`: `Grid.Column` 0, or the last column when reversed; its column is its width.
        GridCell::new(
            SizedBox::new()
                .height(SLIDER_TRACK_THEME_HEIGHT)
                .child(track_rectangle(brushes.decrease)),
        )
        .row(1)
        .column(if reversed { 2 } else { 0 })
        .into_widget(),
    ];
    if placement.shows_top_left() {
        // `TopTickBar`: `VerticalAlignment="Bottom"`, `Margin="0,0,0,4"`, in the pre-content row.
        children.push(
            GridCell::new(
                Padding::new(EdgeInsetsGeometry::from_ltrb(
                    0.0,
                    0.0,
                    0.0,
                    TICK_BAR_MARGIN,
                ))
                .child(
                    Align::new()
                        .alignment(AlignmentGeometry::BOTTOM_CENTER)
                        .child(
                            SizedBox::new()
                                .width(f64::INFINITY)
                                .height(SLIDER_OUTSIDE_TICK_BAR_THEME_HEIGHT)
                                .child(tick_bar(widget, states, brushes.tick_bar)),
                        ),
                ),
            )
            .row(0)
            .column_span(3)
            .into_widget(),
        );
    }
    if placement.shows_inline() {
        // `HorizontalInlineTickBar`: over the track, `SliderInlineTickBarFill`.
        children.push(
            GridCell::new(
                SizedBox::new()
                    .width(f64::INFINITY)
                    .height(SLIDER_TRACK_THEME_HEIGHT)
                    .child(tick_bar(
                        widget,
                        states,
                        resources.slider().slider_inline_tick_bar_fill,
                    )),
            )
            .row(1)
            .column_span(3)
            .into_widget(),
        );
    }
    if placement.shows_bottom_right() {
        // `BottomTickBar`: `VerticalAlignment="Top"`, `Margin="0,4,0,0"`, in the post-content row.
        children.push(
            GridCell::new(
                Padding::new(EdgeInsetsGeometry::from_ltrb(
                    0.0,
                    TICK_BAR_MARGIN,
                    0.0,
                    0.0,
                ))
                .child(
                    Align::new().alignment(AlignmentGeometry::TOP_CENTER).child(
                        SizedBox::new()
                            .width(f64::INFINITY)
                            .height(SLIDER_OUTSIDE_TICK_BAR_THEME_HEIGHT)
                            .child(tick_bar(widget, states, brushes.tick_bar)),
                    ),
                ),
            )
            .row(2)
            .column_span(3)
            .into_widget(),
        );
    }
    // `HorizontalThumb`: column 1, all rows, 18 × 18 centred.
    children.push(
        GridCell::new(
            Center::new()
                .width_factor(1.0)
                .height_factor(1.0)
                .child(thumb(
                    resources,
                    brushes.thumb,
                    states.thumb,
                    SLIDER_HORIZONTAL_THUMB_WIDTH,
                    SLIDER_HORIZONTAL_THUMB_HEIGHT,
                )),
        )
        .row(0)
        .row_span(3)
        .column(1)
        .into_widget(),
    );
    let (first, last) = decrease_definitions(fraction, reversed);
    let grid = Grid::new()
        .column_definitions([
            ColumnDefinition::new(first),
            ColumnDefinition::new(GridLength::AUTO),
            ColumnDefinition::new(last),
        ])
        .row_definitions([
            RowDefinition::new(GridLength::pixel(SLIDER_PRE_CONTENT_MARGIN)),
            RowDefinition::new(GridLength::AUTO),
            RowDefinition::new(GridLength::pixel(SLIDER_POST_CONTENT_MARGIN)),
        ])
        .children(children);
    // `MinHeight="{ThemeResource SliderHorizontalHeight}"`.
    ConstrainedBox::new(BoxConstraints::new().min_height(SLIDER_HORIZONTAL_HEIGHT))
        .child(grid)
        .into_widget()
}

/// `VerticalTemplate`.
fn vertical_template(
    resources: &ThemeResources,
    widget: &Slider,
    states: TemplateStates,
    brushes: &SliderBrushes,
) -> WidgetRef {
    let reversed = widget.is_direction_reversed;
    let fraction = widget.range().fraction_of(states.intermediate_value);
    let placement = widget.tick_placement;
    let mut children = vec![
        // `VerticalTrackRect`.
        GridCell::new(
            SizedBox::new()
                .width(SLIDER_TRACK_THEME_HEIGHT)
                .child(track_rectangle(brushes.track)),
        )
        .column(1)
        .row_span(3)
        .into_widget(),
        // `VerticalDecreaseRect`: `Grid.Row` 2, or row 0 when reversed; its row is its height.
        GridCell::new(
            SizedBox::new()
                .width(SLIDER_TRACK_THEME_HEIGHT)
                .child(track_rectangle(brushes.decrease)),
        )
        .column(1)
        .row(if reversed { 0 } else { 2 })
        .into_widget(),
    ];
    if placement.shows_top_left() {
        // `LeftTickBar`: `HorizontalAlignment="Right"`, `Margin="0,0,4,0"`, in the pre-content column.
        children.push(
            GridCell::new(
                Padding::new(EdgeInsetsGeometry::from_ltrb(
                    0.0,
                    0.0,
                    TICK_BAR_MARGIN,
                    0.0,
                ))
                .child(
                    Align::new()
                        .alignment(AlignmentGeometry::CENTER_RIGHT)
                        .child(
                            SizedBox::new()
                                .width(SLIDER_OUTSIDE_TICK_BAR_THEME_HEIGHT)
                                .height(f64::INFINITY)
                                .child(tick_bar(widget, states, brushes.tick_bar)),
                        ),
                ),
            )
            .column(0)
            .row_span(3)
            .into_widget(),
        );
    }
    if placement.shows_inline() {
        // `VerticalInlineTickBar`.
        children.push(
            GridCell::new(
                SizedBox::new()
                    .width(SLIDER_TRACK_THEME_HEIGHT)
                    .height(f64::INFINITY)
                    .child(tick_bar(
                        widget,
                        states,
                        resources.slider().slider_inline_tick_bar_fill,
                    )),
            )
            .column(1)
            .row_span(3)
            .into_widget(),
        );
    }
    if placement.shows_bottom_right() {
        // `RightTickBar`: `HorizontalAlignment="Left"`, `Margin="4,0,0,0"`, in the post-content column.
        children.push(
            GridCell::new(
                Padding::new(EdgeInsetsGeometry::from_ltrb(
                    TICK_BAR_MARGIN,
                    0.0,
                    0.0,
                    0.0,
                ))
                .child(
                    Align::new()
                        .alignment(AlignmentGeometry::CENTER_LEFT)
                        .child(
                            SizedBox::new()
                                .width(SLIDER_OUTSIDE_TICK_BAR_THEME_HEIGHT)
                                .height(f64::INFINITY)
                                .child(tick_bar(widget, states, brushes.tick_bar)),
                        ),
                ),
            )
            .column(2)
            .row_span(3)
            .into_widget(),
        );
    }
    // `VerticalThumb`: row 1, all columns, 18 × 18 centred.
    children.push(
        GridCell::new(
            Center::new()
                .width_factor(1.0)
                .height_factor(1.0)
                .child(thumb(
                    resources,
                    brushes.thumb,
                    states.thumb,
                    SLIDER_VERTICAL_THUMB_WIDTH,
                    SLIDER_VERTICAL_THUMB_HEIGHT,
                )),
        )
        .row(1)
        .column(0)
        .column_span(3)
        .into_widget(),
    );
    // The decrease rect is in the last row unless reversed.
    let (last, first) = decrease_definitions(fraction, reversed);
    let grid = Grid::new()
        .row_definitions([
            RowDefinition::new(first),
            RowDefinition::new(GridLength::AUTO),
            RowDefinition::new(last),
        ])
        .column_definitions([
            ColumnDefinition::new(GridLength::pixel(SLIDER_PRE_CONTENT_MARGIN)),
            ColumnDefinition::new(GridLength::AUTO),
            ColumnDefinition::new(GridLength::pixel(SLIDER_POST_CONTENT_MARGIN)),
        ])
        .children(children);
    // `MinWidth="{ThemeResource SliderVerticalWidth}"`.
    ConstrainedBox::new(BoxConstraints::new().min_width(SLIDER_VERTICAL_WIDTH))
        .child(grid)
        .into_widget()
}

/// A `Thumb` with `SliderThumbStyle`: the `Border` overhanging the `width × height` slot by `THUMB_BORDER_OVERHANG`, `SliderOuterThumbBackground` inside `SliderThumbBorderBrush`, around `SliderInnerThumb` at the scale its state animates to.
fn thumb(
    resources: &ThemeResources,
    fill: Color,
    state: CommonState,
    width: f64,
    height: f64,
) -> WidgetRef {
    let r = resources.slider();
    let (scale, duration) = inner_thumb_scale(state);
    // `SliderInnerThumb`: `Fill="{TemplateBinding Background}"`, scaled about its centre.
    let inner = AnimatedContainer::new(duration)
        .curve(fast_out_slow_in())
        .width(SLIDER_INNER_THUMB_WIDTH * scale)
        .height(SLIDER_INNER_THUMB_HEIGHT * scale)
        .child(CustomPaint::new().painter(EllipsePainter { fill }));
    let border = ControlBorder::new(
        Brush::Solid(r.slider_outer_thumb_background),
        Brush::ControlElevation(r.slider_thumb_border_brush),
    )
    .border_thickness(THUMB_BORDER_THICKNESS)
    .corner_radius(SLIDER_THUMB_CORNER_RADIUS[0])
    .child(Center::new().child(inner));
    let overhang = 2.0 * THUMB_BORDER_OVERHANG;
    SizedBox::new()
        .width(width)
        .height(height)
        .child(
            OverflowBox::new()
                .min_width(width + overhang)
                .max_width(width + overhang)
                .min_height(height + overhang)
                .max_height(height + overhang)
                .child(border),
        )
        .into_widget()
}

/// XAML `Ellipse` with a solid `Fill`, filling its box.
#[derive(Clone, Debug, PartialEq)]
struct EllipsePainter {
    fill: Color,
}

impl CustomPainter for EllipsePainter {
    fn paint(&self, _app: &mut App, canvas: &mut Canvas, size: Size) {
        let bounds = Rect::from_ltwh(0.0, 0.0, size.width(), size.height());
        draw_oval(canvas, bounds, &Brush::Solid(self.fill).fill(bounds));
    }

    fn should_repaint(&self, _app: &App, old_delegate: &dyn CustomPainter) -> bool {
        old_delegate.as_any().downcast_ref::<Self>() != Some(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
