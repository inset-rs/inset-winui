//! XAML `ToggleButton` with the `DefaultToggleButtonStyle` of `ToggleButton_themeresources.xaml`.
//!
//! The template is one `ContentPresenter` (`x:Name="ContentPresenter"`) carrying `Background`, `BorderBrush`, `BorderThickness`, `CornerRadius`, `Padding`, `Foreground` and `BackgroundSizing`, with a `BrushTransition` of 83 ms on `Background`. `CommonStates` has twelve states: `Normal`, `PointerOver`, `Pressed` and `Disabled`, their `Checked*` twins (which also set `BackgroundSizing` to `ToggleButtonCheckedStateBackgroundSizing`) and their `Indeterminate*` twins; `ToggleButton::ChangeVisualState` (`ToggleButton_Partial.cpp`) picks the row from `IsChecked` and the column from the pointer.

use crate::{
    BUTTON_PADDING, BackgroundSizing, Brush, CONTROL_CONTENT_FONT_SIZE, CONTROL_CORNER_RADIUS,
    CONTROL_FASTER_ANIMATION_DURATION, ColorTransition, CommonState, CommonStates, ControlBorder,
    ControlStates, FocusVisual, TOGGLE_BUTTON_BORDER_THEME_THICKNESS,
    TOGGLE_BUTTON_CHECKED_STATE_BACKGROUND_SIZING, ThemeResources, ToggleButtonResources,
    control_text_style,
};
use reveal_embedder::{Color, FontWeight};
use reveal_foundation::{App, Listener};
use reveal_widgets::*;
use std::{fmt, rc::Rc};

/// XAML `ToggleButtonBorderThemeThickness`.
pub const TOGGLE_BUTTON_BORDER_THICKNESS: f64 = TOGGLE_BUTTON_BORDER_THEME_THICKNESS[0];
/// The `FocusVisualMargin` the style sets.
pub const TOGGLE_BUTTON_FOCUS_VISUAL_MARGIN: [f64; 4] = [-3.0; 4];

/// The brushes the style resolves to in one of its twelve states.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleButtonBrushes {
    pub background: Color,
    pub border: Brush,
    pub foreground: Color,
    pub background_sizing: BackgroundSizing,
}

/// The `{ThemeResource …}` values the `CommonStates` storyboards name for `is_checked` (`Some(true)` the `Checked*` row, `None` the `Indeterminate*` row) and `state`.
pub fn toggle_button_brushes(
    resources: &ToggleButtonResources,
    is_checked: Option<bool>,
    state: CommonState,
) -> ToggleButtonBrushes {
    let r = resources;
    let unchecked = |background, border, foreground| ToggleButtonBrushes {
        background,
        border,
        foreground,
        background_sizing: BackgroundSizing::InnerBorderEdge,
    };
    let checked = |background, border, foreground| ToggleButtonBrushes {
        background,
        border,
        foreground,
        background_sizing: TOGGLE_BUTTON_CHECKED_STATE_BACKGROUND_SIZING,
    };
    match (is_checked, state) {
        // `Normal`: the style's own setters.
        (Some(false), CommonState::Normal) => unchecked(
            r.toggle_button_background,
            Brush::ControlElevation(r.toggle_button_border_brush),
            r.toggle_button_foreground,
        ),
        (Some(false), CommonState::PointerOver) => unchecked(
            r.toggle_button_background_pointer_over,
            Brush::ControlElevation(r.toggle_button_border_brush_pointer_over),
            r.toggle_button_foreground_pointer_over,
        ),
        (Some(false), CommonState::Pressed) => unchecked(
            r.toggle_button_background_pressed,
            Brush::Solid(r.toggle_button_border_brush_pressed),
            r.toggle_button_foreground_pressed,
        ),
        (Some(false), CommonState::Disabled) => unchecked(
            r.toggle_button_background_disabled,
            Brush::Solid(r.toggle_button_border_brush_disabled),
            r.toggle_button_foreground_disabled,
        ),
        (Some(true), CommonState::Normal) => checked(
            r.toggle_button_background_checked,
            Brush::ControlElevation(r.toggle_button_border_brush_checked),
            r.toggle_button_foreground_checked,
        ),
        (Some(true), CommonState::PointerOver) => checked(
            r.toggle_button_background_checked_pointer_over,
            Brush::ControlElevation(r.toggle_button_border_brush_checked_pointer_over),
            r.toggle_button_foreground_checked_pointer_over,
        ),
        (Some(true), CommonState::Pressed) => checked(
            r.toggle_button_background_checked_pressed,
            Brush::Solid(r.toggle_button_border_brush_checked_pressed),
            r.toggle_button_foreground_checked_pressed,
        ),
        // `CheckedDisabled` names no `BackgroundSizing`, so the style's `InnerBorderEdge` holds.
        (Some(true), CommonState::Disabled) => unchecked(
            r.toggle_button_background_checked_disabled,
            Brush::Solid(r.toggle_button_border_brush_checked_disabled),
            r.toggle_button_foreground_checked_disabled,
        ),
        (None, CommonState::Normal) => unchecked(
            r.toggle_button_background_indeterminate,
            Brush::ControlElevation(r.toggle_button_border_brush_indeterminate),
            r.toggle_button_foreground_indeterminate,
        ),
        (None, CommonState::PointerOver) => unchecked(
            r.toggle_button_background_indeterminate_pointer_over,
            Brush::ControlElevation(r.toggle_button_border_brush_indeterminate_pointer_over),
            r.toggle_button_foreground_indeterminate_pointer_over,
        ),
        (None, CommonState::Pressed) => unchecked(
            r.toggle_button_background_indeterminate_pressed,
            Brush::Solid(r.toggle_button_border_brush_indeterminate_pressed),
            r.toggle_button_foreground_indeterminate_pressed,
        ),
        (None, CommonState::Disabled) => unchecked(
            r.toggle_button_background_indeterminate_disabled,
            Brush::Solid(r.toggle_button_border_brush_indeterminate_disabled),
            r.toggle_button_foreground_indeterminate_disabled,
        ),
    }
}

/// The value `IsChecked` takes after a toggle; `ToggleButton::OnToggleImpl`: indeterminate becomes unchecked, unchecked becomes checked, and checked becomes indeterminate when `IsThreeState` and unchecked otherwise.
pub fn next_is_checked(is_checked: Option<bool>, is_three_state: bool) -> Option<bool> {
    match is_checked {
        None => Some(false),
        Some(true) if is_three_state => None,
        Some(true) => Some(false),
        Some(false) => Some(true),
    }
}

/// The `Checked`, `Unchecked` and `Indeterminate` events as one handler, given the value the button wants.
pub type ToggleButtonCheckedHandler = Rc<dyn Fn(&mut App, Option<bool>)>;

/// Builds a source ToggleButton template while retaining native toggle and keyboard behavior.
pub type ToggleButtonTemplate =
    Rc<dyn Fn(&mut App, BuildContext, Option<bool>, Option<WidgetRef>, ControlStates) -> WidgetRef>;

/// XAML `ToggleButton`: `IsChecked` (`Some(true)`, `Some(false)` or `None` for indeterminate) with `IsThreeState`, `Content`, and `Click` after each toggle.
#[derive(Clone)]
pub struct ToggleButton {
    pub is_checked: Option<bool>,
    /// XAML `Checked` / `Unchecked` / `Indeterminate`, with the value the button wants.
    pub checked: ToggleButtonCheckedHandler,
    pub content: Option<WidgetRef>,
    /// XAML `IsThreeState`: whether a toggle from checked goes through indeterminate.
    pub is_three_state: bool,
    /// XAML `Click`, raised after the toggle as `ToggleButton::OnClick` does.
    pub click: Option<Listener>,
    pub is_enabled: bool,

    /// Optional source template used by compound controls such as Expander.
    pub template: Option<ToggleButtonTemplate>,

    pub key: Option<KeyRef>,
}

impl ToggleButton {
    pub fn new(
        is_checked: Option<bool>,
        checked: impl Fn(&mut App, Option<bool>) + 'static,
    ) -> ToggleButton {
        ToggleButton {
            is_checked,
            checked: Rc::new(checked),
            content: None,
            is_three_state: false,
            click: None,
            is_enabled: true,
            template: None,
            key: None,
        }
    }

    /// A text `Content`, as `<ToggleButton Content="…"/>`.
    pub fn text(
        text: impl Into<String>,
        is_checked: Option<bool>,
        checked: impl Fn(&mut App, Option<bool>) + 'static,
    ) -> ToggleButton {
        ToggleButton::new(is_checked, checked).content(Text::new(text))
    }

    /// XAML `Content`.
    pub fn content<K>(mut self, content: impl IntoWidget<K>) -> ToggleButton {
        self.content = Some(content.into_widget());
        self
    }

    /// XAML `IsThreeState`.
    pub fn is_three_state(mut self, is_three_state: bool) -> ToggleButton {
        self.is_three_state = is_three_state;
        self
    }

    /// XAML `Click`.
    pub fn click(mut self, click: Listener) -> ToggleButton {
        self.click = Some(click);
        self
    }

    /// XAML `IsEnabled`.
    pub fn is_enabled(mut self, enabled: bool) -> ToggleButton {
        self.is_enabled = enabled;
        self
    }

    /// Sets an alternate ControlTemplate without changing the toggle behavior.
    pub fn template(
        mut self,
        builder: impl Fn(
            &mut App,
            BuildContext,
            Option<bool>,
            Option<WidgetRef>,
            ControlStates,
        ) -> WidgetRef
        + 'static,
    ) -> Self {
        self.template = Some(Rc::new(builder));
        self
    }

    pub fn key(mut self, key: KeyRef) -> ToggleButton {
        self.key = Some(key);
        self
    }
}

impl fmt::Debug for ToggleButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToggleButton")
            .field("is_checked", &self.is_checked)
            .field("is_three_state", &self.is_three_state)
            .field("is_enabled", &self.is_enabled)
            .finish_non_exhaustive()
    }
}

impl StatelessWidget for ToggleButton {
    fn key(&self) -> Option<&KeyRef> {
        self.key.as_ref()
    }

    fn build(&self, _app: &mut App, _context: BuildContext) -> WidgetRef {
        let (is_checked, content) = (self.is_checked, self.content.clone());
        let next = next_is_checked(is_checked, self.is_three_state);
        let (checked, click) = (self.checked.clone(), self.click.clone());
        // `ToggleButton::OnClick`: `OnToggle`, then the base `Click`.
        let on_click = Listener::new(move |app| {
            checked(app, next);
            if let Some(click) = &click {
                click.call(app);
            }
        });
        let custom_template = self.template.clone();
        CommonStates::new(on_click, move |app, context, states| {
            if let Some(builder) = &custom_template {
                builder(app, context, is_checked, content.clone(), states)
            } else {
                template(app, context, is_checked, content.clone(), states)
            }
        })
        .is_enabled(self.is_enabled)
        .into_widget()
    }
}

/// The `ControlTemplate` of `DefaultToggleButtonStyle` for one `IsChecked` and set of states.
fn template(
    app: &mut App,
    context: BuildContext,
    is_checked: Option<bool>,
    content: Option<WidgetRef>,
    states: ControlStates,
) -> WidgetRef {
    let resources = ThemeResources::of(app, context);
    let brushes = toggle_button_brushes(&resources.toggle_button(), is_checked, states.common);
    let text = control_text_style(
        CONTROL_CONTENT_FONT_SIZE,
        FontWeight::W400,
        brushes.foreground,
    );
    let content = content.unwrap_or_else(|| SizedBox::shrink().into_widget());
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
                .border_thickness(TOGGLE_BUTTON_BORDER_THICKNESS)
                .corner_radius(CONTROL_CORNER_RADIUS[0])
                .background_sizing(brushes.background_sizing)
                .padding(BUTTON_PADDING)
                .child(label.clone())
                .into_widget()
        },
    );
    FocusVisual::new(presenter, resources.theme)
        .visible(states.focused)
        .margin(TOGGLE_BUTTON_FOCUS_VISUAL_MARGIN)
        .corner_radius(CONTROL_CORNER_RADIUS[0])
        .into_widget()
}
