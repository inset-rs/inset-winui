//! One navigation entry per feature, following the Cupertino gallery's explicit catalog.

use reveal_winui::FluentSymbol;

/// A gallery destination with its own retained examples and scroll position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feature {
    /// Standard, accent and disabled buttons.
    Button,
    /// Switch content, headers and dragging.
    ToggleSwitch,
    /// Pixel, auto and star layout.
    Grid,
    /// Two-state and three-state check boxes.
    CheckBox,
    /// Mutually exclusive choices.
    RadioButton,
    /// Buttons that retain a checked state.
    ToggleButton,
    /// Repeated activation while held.
    RepeatButton,
    /// Link presentation and activation.
    HyperlinkButton,
    /// Continuous and stepped values.
    Slider,
    /// Inline and overlay panes.
    SplitView,
    /// Backdrop blur and tint with sharp foreground content.
    Acrylic,
    /// Pointer and keyboard help.
    ToolTip,
    /// Tab selection, closing and dragging.
    TabView,
    /// The available text styles and foreground hierarchy.
    Typography,
    /// Fluent symbol sizes, colors and mirroring.
    Icons,
    /// Adaptive panes, hierarchy and top navigation.
    NavigationView,
}

impl Feature {
    /// Display order in the navigation pane.
    pub const ALL: [Self; 16] = [
        Self::Button,
        Self::ToggleSwitch,
        Self::Grid,
        Self::CheckBox,
        Self::RadioButton,
        Self::ToggleButton,
        Self::RepeatButton,
        Self::HyperlinkButton,
        Self::Slider,
        Self::SplitView,
        Self::Acrylic,
        Self::ToolTip,
        Self::TabView,
        Self::NavigationView,
        Self::Typography,
        Self::Icons,
    ];

    /// The navigation label and stable destination identity.
    pub fn title(self) -> &'static str {
        match self {
            Self::Button => "Button",
            Self::ToggleSwitch => "ToggleSwitch",
            Self::Grid => "Grid",
            Self::CheckBox => "CheckBox",
            Self::RadioButton => "RadioButton",
            Self::ToggleButton => "ToggleButton",
            Self::RepeatButton => "RepeatButton",
            Self::HyperlinkButton => "HyperlinkButton",
            Self::Slider => "Slider",
            Self::SplitView => "SplitView",
            Self::Acrylic => "Acrylic",
            Self::ToolTip => "ToolTip",
            Self::TabView => "TabView",
            Self::NavigationView => "NavigationView",
            Self::Typography => "Typography",
            Self::Icons => "Icons",
        }
    }

    /// A distinct bundled Fluent symbol keeps each destination recognizable in compact mode.
    pub fn symbol(self) -> FluentSymbol {
        match self {
            Self::Button => FluentSymbol::CursorClick,
            Self::ToggleSwitch => FluentSymbol::ToggleLeft,
            Self::Grid => FluentSymbol::Grid,
            Self::CheckBox => FluentSymbol::CheckboxChecked,
            Self::RadioButton => FluentSymbol::RadioButton,
            Self::ToggleButton => FluentSymbol::CheckboxIndeterminate,
            Self::RepeatButton => FluentSymbol::ArrowRepeatAll,
            Self::HyperlinkButton => FluentSymbol::Link,
            Self::Slider => FluentSymbol::Options,
            Self::SplitView => FluentSymbol::PanelLeft,
            Self::Acrylic => FluentSymbol::Layer,
            Self::ToolTip => FluentSymbol::TooltipQuote,
            Self::TabView => FluentSymbol::Tab,
            Self::NavigationView => FluentSymbol::Navigation,
            Self::Typography => FluentSymbol::TextFont,
            Self::Icons => FluentSymbol::Apps,
        }
    }

    /// Brief guidance above the examples.
    pub fn description(self) -> &'static str {
        match self {
            Self::Button => "Activate an action using the pointer or keyboard.",
            Self::ToggleSwitch => "Switch settings on and off, or drag the thumb.",
            Self::Grid => "Arrange content with fixed, content-sized and proportional tracks.",
            Self::CheckBox => "Choose independent options, including an indeterminate state.",
            Self::RadioButton => "Choose one option from a group.",
            Self::ToggleButton => "Keep an action checked until it is toggled again.",
            Self::RepeatButton => "Hold the button to repeat its action.",
            Self::HyperlinkButton => "Present an action as a text link.",
            Self::Slider => "Adjust a value continuously or in steps.",
            Self::SplitView => "Explore inline, compact and overlay pane layouts.",
            Self::Acrylic => {
                "Blur and tint the scene behind a surface while keeping its content sharp."
            }
            Self::ToolTip => "Hover or focus a control to reveal its help text.",
            Self::TabView => "Add, close, reorder and move tabs between strips.",
            Self::NavigationView => "Explore pane modes, nested destinations and top overflow.",
            Self::Typography => "Choose text styles and build a clear hierarchy.",
            Self::Icons => "Use Fluent symbols in navigation, actions and status indicators.",
        }
    }
}
