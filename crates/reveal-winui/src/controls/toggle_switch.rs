//! XAML `ToggleSwitch` with the Fluent style of `ToggleSwitch_themeresources.xaml`.
//!
//! Template, top to bottom: `HeaderContentPresenter`; a grid whose rows are `ToggleSwitchPreContentMargin`, the switch and `ToggleSwitchPostContentMargin` and whose columns are the switch, a 12 pt gap and the `OffContentPresenter` / `OnContentPresenter`; `SwitchAreaGrid` (`Margin="0,5"`) holds `OuterBorder` (40 × 20, radius 10, the off fill and stroke), `SwitchKnobBounds` (the on fill, shown in the `On` state) and `SwitchKnob` (a 20 × 20 grid translated to the right end when on) containing `SwitchKnobOn` (a 12 pt `Border` with the circle elevation stroke) and `SwitchKnobOff` (a 12 pt circle).

use crate::{
    BackgroundSizing, Brush, CONTROL_CONTENT_FONT_SIZE, CONTROL_CORNER_RADIUS,
    CONTROL_FAST_ANIMATION_DURATION, CONTROL_FAST_OUT_SLOW_IN_KEY_SPLINE,
    CONTROL_FASTER_ANIMATION_DURATION, CommonState, CommonStates, ControlBorder, ControlStates,
    FocusVisual, TOGGLE_SWITCH_POST_CONTENT_MARGIN, TOGGLE_SWITCH_PRE_CONTENT_MARGIN,
    TOGGLE_SWITCH_THEME_MIN_WIDTH, TOGGLE_SWITCH_TOP_HEADER_MARGIN, ThemeResources,
    ToggleSwitchResources, control_text_style,
};
use reveal_animation::{Cubic, Curve};
use reveal_embedder::{Color, FontWeight};
use reveal_foundation::{App, Listener};
use reveal_painting::EdgeInsetsGeometry;
use reveal_rendering::{BoxConstraints, CrossAxisAlignment, MainAxisSize};
use reveal_widgets::*;
use std::{fmt, rc::Rc, time::Duration};

/// `OuterBorder` and `SwitchKnobBounds`: the track.
pub const TRACK_WIDTH: f64 = 40.0;
pub const TRACK_HEIGHT: f64 = 20.0;
/// `SwitchKnob`: the grid the knob shapes centre in; it travels `TRACK_WIDTH − KNOB_AREA` when toggled.
pub const KNOB_AREA: f64 = 20.0;
/// `ToggleSwitchOuterBorderStrokeThickness`.
pub const OUTER_BORDER_STROKE_THICKNESS: f64 = 1.0;
/// The gap column between the switch and its content.
pub const CONTENT_GAP: f64 = 12.0;
/// `FocusVisualMargin` of the style.
pub const FOCUS_VISUAL_MARGIN: [f64; 4] = [-7.0, -3.0, -7.0, -3.0];

/// The knob's size per common state: the storyboards animate `SwitchKnobOn` and `SwitchKnobOff` to 12 × 12 (`Normal`), 14 × 14 (`PointerOver`) and 17 × 14 (`Pressed`) over `ControlFasterAnimationDuration` with `ControlFastOutSlowInKeySpline`.
pub fn knob_size(state: CommonState) -> (f64, f64) {
    match state {
        CommonState::Normal | CommonState::Disabled => (12.0, 12.0),
        CommonState::PointerOver => (14.0, 14.0),
        CommonState::Pressed => (17.0, 14.0),
    }
}

/// XAML `ControlFastOutSlowInKeySpline` as a curve.
pub fn fast_out_slow_in() -> Rc<dyn Curve> {
    let [x1, y1, x2, y2] = CONTROL_FAST_OUT_SLOW_IN_KEY_SPLINE;
    Rc::new(Cubic::new(x1, y1, x2, y2))
}

/// How long the knob takes to travel between the ends. XAML animates it with `RepositionThemeAnimation`, whose timing lives in the OS animation library rather than in the source; `ControlFastAnimationDuration` stands in.
pub const KNOB_TRAVEL_DURATION: Duration = CONTROL_FAST_ANIMATION_DURATION;

/// XAML `Toggled` handler, given the value the switch wants.
pub type ToggledHandler = Rc<dyn Fn(&mut App, bool)>;

/// XAML `ToggleSwitch`: `IsOn` with `Toggled`, `Header`, `OnContent` and `OffContent`.
#[derive(Clone)]
pub struct ToggleSwitch {
    pub is_on: bool,
    /// XAML `Toggled`, with the value the switch wants.
    pub toggled: ToggledHandler,
    pub header: Option<WidgetRef>,
    /// XAML `OnContent`; the control's default is the text "On".
    pub on_content: Option<WidgetRef>,
    /// XAML `OffContent`; the control's default is the text "Off".
    pub off_content: Option<WidgetRef>,
    pub is_enabled: bool,
    pub key: Option<KeyRef>,
}

impl ToggleSwitch {
    pub fn new(is_on: bool, toggled: impl Fn(&mut App, bool) + 'static) -> ToggleSwitch {
        ToggleSwitch {
            is_on,
            toggled: Rc::new(toggled),
            header: None,
            on_content: None,
            off_content: None,
            is_enabled: true,
            key: None,
        }
    }

    pub fn header<K>(mut self, header: impl IntoWidget<K>) -> ToggleSwitch {
        self.header = Some(header.into_widget());
        self
    }

    pub fn on_content<K>(mut self, content: impl IntoWidget<K>) -> ToggleSwitch {
        self.on_content = Some(content.into_widget());
        self
    }

    pub fn off_content<K>(mut self, content: impl IntoWidget<K>) -> ToggleSwitch {
        self.off_content = Some(content.into_widget());
        self
    }

    pub fn is_enabled(mut self, enabled: bool) -> ToggleSwitch {
        self.is_enabled = enabled;
        self
    }

    pub fn key(mut self, key: KeyRef) -> ToggleSwitch {
        self.key = Some(key);
        self
    }
}

impl fmt::Debug for ToggleSwitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToggleSwitch")
            .field("is_on", &self.is_on)
            .field("is_enabled", &self.is_enabled)
            .finish_non_exhaustive()
    }
}

impl StatelessWidget for ToggleSwitch {
    fn key(&self) -> Option<&KeyRef> {
        self.key.as_ref()
    }

    fn build(&self, _app: &mut App, _context: BuildContext) -> WidgetRef {
        let widget = self.clone();
        let toggled = self.toggled.clone();
        let is_on = self.is_on;
        let click = Listener::new(move |app| toggled(app, !is_on));
        CommonStates::new(click, move |app, context, states| {
            template(app, context, &widget, states)
        })
        .is_enabled(self.is_enabled)
        .into_widget()
    }
}

/// The track, knob and content brushes for one state; the `{ThemeResource …}` values the `CommonStates` storyboards name.
struct SwitchBrushes {
    fill_off: Color,
    stroke_off: Color,
    fill_on: Color,
    stroke_on: Color,
    knob_off: Color,
    knob_on: Color,
    container: Color,
    foreground: Color,
    header: Color,
}

fn brushes(r: &ToggleSwitchResources, state: CommonState) -> SwitchBrushes {
    match state {
        CommonState::Normal => SwitchBrushes {
            fill_off: r.toggle_switch_fill_off,
            stroke_off: r.toggle_switch_stroke_off,
            fill_on: r.toggle_switch_fill_on,
            stroke_on: r.toggle_switch_stroke_on,
            knob_off: r.toggle_switch_knob_fill_off,
            knob_on: r.toggle_switch_knob_fill_on,
            container: r.toggle_switch_container_background,
            foreground: r.toggle_switch_content_foreground,
            header: r.toggle_switch_header_foreground,
        },
        CommonState::PointerOver => SwitchBrushes {
            fill_off: r.toggle_switch_fill_off_pointer_over,
            stroke_off: r.toggle_switch_stroke_off_pointer_over,
            fill_on: r.toggle_switch_fill_on_pointer_over,
            stroke_on: r.toggle_switch_stroke_on_pointer_over,
            knob_off: r.toggle_switch_knob_fill_off_pointer_over,
            knob_on: r.toggle_switch_knob_fill_on_pointer_over,
            container: r.toggle_switch_container_background_pointer_over,
            foreground: r.toggle_switch_content_foreground,
            header: r.toggle_switch_header_foreground,
        },
        CommonState::Pressed => SwitchBrushes {
            fill_off: r.toggle_switch_fill_off_pressed,
            stroke_off: r.toggle_switch_stroke_off_pressed,
            fill_on: r.toggle_switch_fill_on_pressed,
            stroke_on: r.toggle_switch_stroke_on_pressed,
            knob_off: r.toggle_switch_knob_fill_off_pressed,
            knob_on: r.toggle_switch_knob_fill_on_pressed,
            container: r.toggle_switch_container_background_pressed,
            foreground: r.toggle_switch_content_foreground,
            header: r.toggle_switch_header_foreground,
        },
        CommonState::Disabled => SwitchBrushes {
            fill_off: r.toggle_switch_fill_off_disabled,
            stroke_off: r.toggle_switch_stroke_off_disabled,
            fill_on: r.toggle_switch_fill_on_disabled,
            stroke_on: r.toggle_switch_stroke_on_disabled,
            knob_off: r.toggle_switch_knob_fill_off_disabled,
            knob_on: r.toggle_switch_knob_fill_on_disabled,
            container: r.toggle_switch_container_background_disabled,
            foreground: r.toggle_switch_content_foreground_disabled,
            header: r.toggle_switch_header_foreground_disabled,
        },
    }
}

fn template(
    app: &mut App,
    context: BuildContext,
    widget: &ToggleSwitch,
    states: ControlStates,
) -> WidgetRef {
    let resources = ThemeResources::of(app, context);
    let r = resources.toggle_switch();
    let brushes = brushes(&r, states.common);
    let text = control_text_style(
        CONTROL_CONTENT_FONT_SIZE,
        FontWeight::W400,
        brushes.foreground,
    );
    let switch = switch_area(&resources, &brushes, widget.is_on, states.common);
    // `OffContentPresenter` / `OnContentPresenter`: the `ContentStates` group shows one of them.
    let content = if widget.is_on {
        widget
            .on_content
            .clone()
            .unwrap_or_else(|| Text::new("On").into_widget())
    } else {
        widget
            .off_content
            .clone()
            .unwrap_or_else(|| Text::new("Off").into_widget())
    };
    let row = Row::new()
        .main_axis_size(MainAxisSize::Min)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(vec![
            switch,
            SizedBox::new().width(CONTENT_GAP).into_widget(),
            DefaultTextStyle::new(text, content).into_widget(),
        ]);
    let body = ConstrainedBox::new(BoxConstraints::new().min_width(TOGGLE_SWITCH_THEME_MIN_WIDTH))
        .child(
            Padding::new(EdgeInsetsGeometry::symmetric(
                TOGGLE_SWITCH_PRE_CONTENT_MARGIN.min(TOGGLE_SWITCH_POST_CONTENT_MARGIN),
                0.0,
            ))
            .child(row),
        )
        .into_widget();
    let body = match &widget.header {
        Some(header) => {
            let [left, top, right, bottom] = TOGGLE_SWITCH_TOP_HEADER_MARGIN;
            let header_text =
                control_text_style(CONTROL_CONTENT_FONT_SIZE, FontWeight::W400, brushes.header);
            Column::new()
                .main_axis_size(MainAxisSize::Min)
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .children(vec![
                    Padding::new(EdgeInsetsGeometry::from_ltrb(left, top, right, bottom))
                        .child(DefaultTextStyle::new(header_text, header.clone()))
                        .into_widget(),
                    body,
                ])
                .into_widget()
        }
        None => body,
    };
    FocusVisual::new(body, resources.theme)
        .visible(states.focused)
        .margin(FOCUS_VISUAL_MARGIN)
        .corner_radius(CONTROL_CORNER_RADIUS[0])
        .into_widget()
}

/// `SwitchAreaGrid` with the track and the knob.
fn switch_area(
    resources: &ThemeResources,
    brushes: &SwitchBrushes,
    is_on: bool,
    state: CommonState,
) -> WidgetRef {
    let (knob_width, knob_height) = knob_size(state);
    let track = if is_on {
        // `SwitchKnobBounds`: `ToggleSwitchOnStrokeThickness` is 0.
        ControlBorder::new(
            Brush::Solid(brushes.fill_on),
            Brush::Solid(brushes.stroke_on),
        )
        .border_thickness(0.0)
    } else {
        // `OuterBorder`.
        ControlBorder::new(
            Brush::Solid(brushes.fill_off),
            Brush::Solid(brushes.stroke_off),
        )
        .border_thickness(OUTER_BORDER_STROKE_THICKNESS)
    }
    .corner_radius(TRACK_HEIGHT / 2.0);
    let knob = if is_on {
        // `SwitchKnobOn`: the elevation-stroked accent knob, `BackgroundSizing="OuterBorderEdge"`.
        ControlBorder::new(
            Brush::Solid(brushes.knob_on),
            Brush::CircleElevation(resources.toggle_switch().toggle_switch_knob_stroke_on),
        )
        .border_thickness(1.0)
        .background_sizing(BackgroundSizing::OuterBorderEdge)
        .corner_radius(7.0)
        .into_widget()
    } else {
        // `SwitchKnobOff`.
        ControlBorder::new(
            Brush::Solid(brushes.knob_off),
            Brush::Solid(Color::from_argb(0, 0, 0, 0)),
        )
        .border_thickness(0.0)
        .corner_radius(7.0)
        .into_widget()
    };
    let knob = AnimatedContainer::new(CONTROL_FASTER_ANIMATION_DURATION)
        .curve(fast_out_slow_in())
        .width(knob_width)
        .height(knob_height)
        .child(knob);
    // A pressed knob stretches toward its destination: the storyboards set `HorizontalAlignment`
    // `Right` on `SwitchKnobOn` and `Left` on `SwitchKnobOff`.
    let knob_left = (KNOB_AREA - knob_width) / 2.0
        + match (state, is_on) {
            (CommonState::Pressed, true) => (knob_width - 12.0) / 2.0,
            (CommonState::Pressed, false) => -(knob_width - 12.0) / 2.0,
            _ => 0.0,
        }
        + if is_on { TRACK_WIDTH - KNOB_AREA } else { 0.0 };
    let knob_top = (TRACK_HEIGHT - knob_height) / 2.0;
    let area = SizedBox::new()
        .width(TRACK_WIDTH)
        .height(TRACK_HEIGHT)
        .child(Stack::new().children(vec![
            Positioned::fill(track).into_widget(),
            AnimatedPositioned::new(knob, KNOB_TRAVEL_DURATION)
                .curve(fast_out_slow_in())
                .left(knob_left)
                .top(knob_top)
                .width(knob_width)
                .height(knob_height)
                .into_widget(),
        ]))
        .into_widget();
    // `SwitchAreaGrid`: `Margin="0,5"` with the container background.
    ColoredBox::new(brushes.container)
        .child(Padding::new(EdgeInsetsGeometry::symmetric(5.0, 0.0)).child(area))
        .into_widget()
}
