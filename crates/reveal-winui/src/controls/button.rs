//! XAML `Button` with the `DefaultButtonStyle`, `AccentButtonStyle` and `SubtleButtonStyle` of `Button_themeresources.xaml`.
//!
//! The template is one `ContentPresenter` (`x:Name="ContentPresenter"`) carrying `Background`, `BorderBrush`, `BorderThickness`, `CornerRadius`, `Padding` and `Foreground`, with a `BrushTransition` of 83 ms on `Background`; the `CommonStates` storyboards swap those brushes per state.

use crate::{
    BUTTON_PADDING, BackgroundSizing, Brush, ButtonResources, CONTROL_CONTENT_FONT_SIZE,
    CONTROL_CORNER_RADIUS, CONTROL_FASTER_ANIMATION_DURATION, ColorTransition, CommonState,
    CommonStates, ControlBorder, ControlStates, FocusVisual, ThemeResources, control_text_style,
};
use reveal_embedder::{Color, FontWeight};
use reveal_foundation::{App, Listener};
use reveal_widgets::*;
use std::fmt;

/// XAML `ButtonBorderThemeThickness`.
pub const BUTTON_BORDER_THICKNESS: f64 = 1.0;
/// The `FocusVisualMargin` the button styles set.
pub const BUTTON_FOCUS_VISUAL_MARGIN: [f64; 4] = [-3.0; 4];

/// Which `Button` style the control uses; XAML `Style="{StaticResource AccentButtonStyle}"` and its siblings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonStyle {
    /// `DefaultButtonStyle`.
    #[default]
    Default,
    /// `AccentButtonStyle`: the accent fill, `BackgroundSizing="OuterBorderEdge"`.
    Accent,
    /// `SubtleButtonStyle`: no fill or border until the pointer is over it.
    Subtle,
}

/// The brushes a button style resolves to in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonBrushes {
    pub background: Color,
    pub border: Brush,
    pub foreground: Color,
}

impl ButtonStyle {
    /// The `{ThemeResource …}` values the style's `CommonStates` storyboards name for `state`.
    pub fn brushes(self, resources: &ButtonResources, state: CommonState) -> ButtonBrushes {
        let r = resources;
        match (self, state) {
            (ButtonStyle::Default, CommonState::Normal) => ButtonBrushes {
                background: r.button_background,
                border: Brush::ControlElevation(r.button_border_brush),
                foreground: r.button_foreground,
            },
            (ButtonStyle::Default, CommonState::PointerOver) => ButtonBrushes {
                background: r.button_background_pointer_over,
                border: Brush::ControlElevation(r.button_border_brush_pointer_over),
                foreground: r.button_foreground_pointer_over,
            },
            (ButtonStyle::Default, CommonState::Pressed) => ButtonBrushes {
                background: r.button_background_pressed,
                border: Brush::Solid(r.button_border_brush_pressed),
                foreground: r.button_foreground_pressed,
            },
            (ButtonStyle::Default, CommonState::Disabled) => ButtonBrushes {
                background: r.button_background_disabled,
                border: Brush::Solid(r.button_border_brush_disabled),
                foreground: r.button_foreground_disabled,
            },
            (ButtonStyle::Accent, CommonState::Normal) => ButtonBrushes {
                background: r.accent_button_background,
                border: Brush::ControlElevation(r.accent_button_border_brush),
                foreground: r.accent_button_foreground,
            },
            (ButtonStyle::Accent, CommonState::PointerOver) => ButtonBrushes {
                background: r.accent_button_background_pointer_over,
                border: Brush::ControlElevation(r.accent_button_border_brush_pointer_over),
                foreground: r.accent_button_foreground_pointer_over,
            },
            (ButtonStyle::Accent, CommonState::Pressed) => ButtonBrushes {
                background: r.accent_button_background_pressed,
                border: Brush::Solid(r.accent_button_border_brush_pressed),
                foreground: r.accent_button_foreground_pressed,
            },
            (ButtonStyle::Accent, CommonState::Disabled) => ButtonBrushes {
                background: r.accent_button_background_disabled,
                border: Brush::Solid(r.accent_button_border_brush_disabled),
                foreground: r.accent_button_foreground_disabled,
            },
            (ButtonStyle::Subtle, CommonState::Normal) => ButtonBrushes {
                background: r.subtle_button_background,
                border: Brush::Solid(r.subtle_button_border_brush),
                foreground: r.subtle_button_foreground,
            },
            (ButtonStyle::Subtle, CommonState::PointerOver) => ButtonBrushes {
                background: r.subtle_button_background_pointer_over,
                border: Brush::Solid(r.subtle_button_border_brush_pointer_over),
                foreground: r.subtle_button_foreground_pointer_over,
            },
            (ButtonStyle::Subtle, CommonState::Pressed) => ButtonBrushes {
                background: r.subtle_button_background_pressed,
                border: Brush::Solid(r.subtle_button_border_brush_pressed),
                foreground: r.subtle_button_foreground_pressed,
            },
            (ButtonStyle::Subtle, CommonState::Disabled) => ButtonBrushes {
                background: r.subtle_button_background_disabled,
                border: Brush::Solid(r.subtle_button_border_brush_disabled),
                foreground: r.subtle_button_foreground_disabled,
            },
        }
    }

    /// XAML `BackgroundSizing` of the style.
    pub fn background_sizing(self) -> BackgroundSizing {
        match self {
            ButtonStyle::Accent => BackgroundSizing::OuterBorderEdge,
            ButtonStyle::Default | ButtonStyle::Subtle => BackgroundSizing::InnerBorderEdge,
        }
    }
}

/// XAML `Button`: `Content` shown in the style's bezel, `Click` raised on activation.
#[derive(Clone)]
pub struct Button {
    pub content: WidgetRef,
    pub click: Listener,
    pub style: ButtonStyle,
    pub is_enabled: bool,
    pub key: Option<KeyRef>,
}

impl Button {
    pub fn new<K>(content: impl IntoWidget<K>, click: Listener) -> Button {
        Button {
            content: content.into_widget(),
            click,
            style: ButtonStyle::Default,
            is_enabled: true,
            key: None,
        }
    }

    /// A text `Content`, as `<Button Content="…"/>`.
    pub fn text(text: impl Into<String>, click: Listener) -> Button {
        Button::new(Text::new(text), click)
    }

    pub fn style(mut self, style: ButtonStyle) -> Button {
        self.style = style;
        self
    }

    /// XAML `IsEnabled`.
    pub fn is_enabled(mut self, enabled: bool) -> Button {
        self.is_enabled = enabled;
        self
    }

    pub fn key(mut self, key: KeyRef) -> Button {
        self.key = Some(key);
        self
    }
}

impl fmt::Debug for Button {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Button")
            .field("style", &self.style)
            .field("is_enabled", &self.is_enabled)
            .finish_non_exhaustive()
    }
}

impl StatelessWidget for Button {
    fn key(&self) -> Option<&KeyRef> {
        self.key.as_ref()
    }

    fn build(&self, _app: &mut App, _context: BuildContext) -> WidgetRef {
        let (content, style) = (self.content.clone(), self.style);
        CommonStates::new(self.click.clone(), move |app, context, states| {
            template(app, context, style, content.clone(), states)
        })
        .is_enabled(self.is_enabled)
        .into_widget()
    }
}

/// The `ControlTemplate` of the button styles for one set of states.
fn template(
    app: &mut App,
    context: BuildContext,
    style: ButtonStyle,
    content: WidgetRef,
    states: ControlStates,
) -> WidgetRef {
    let resources = ThemeResources::of(app, context);
    let brushes = style.brushes(&resources.button(), states.common);
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
                .border_thickness(BUTTON_BORDER_THICKNESS)
                .corner_radius(CONTROL_CORNER_RADIUS[0])
                .background_sizing(style.background_sizing())
                .padding(BUTTON_PADDING)
                .child(label.clone())
                .into_widget()
        },
    );
    FocusVisual::new(presenter, resources.theme)
        .visible(states.focused)
        .margin(BUTTON_FOCUS_VISUAL_MARGIN)
        .corner_radius(CONTROL_CORNER_RADIUS[0])
        .into_widget()
}
