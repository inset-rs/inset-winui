//! FontIcon presentation from WinUI `core/elements/icon.cpp`, using the open
//! Fluent System Icons font for the kit's non-Windows symbol substitute.

use reveal_embedder::{Color, FontWeight, TextAlign, TextDirection};
use reveal_foundation::App;
use reveal_painting::{PaintingBinding, TextScaler, TextStyle};
use reveal_widgets::*;

/// The family under which the bundled open icon font is registered.
pub const FLUENT_ICON_FONT_FAMILY: &str = "FluentSystemIcons-Regular";

/// Registers the bundled font before the first icon is laid out.
pub fn install_icon_font(app: &mut App) {
    PaintingBinding::instance(app).register_font(
        app,
        FLUENT_ICON_FONT_FAMILY,
        include_bytes!("../../assets/fluent-system-icons/FluentSystemIcons-Regular.ttf").to_vec(),
    );
}

/// Built-in control symbols, mapped to the open Fluent System Icons codepoints.
/// These names intentionally do not promise Segoe's private-use glyph mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FluentSymbol {
    /// Activate an action.
    CursorClick,
    /// Switch a setting.
    ToggleLeft,
    /// A checked choice.
    CheckboxChecked,
    /// A mixed selection.
    CheckboxIndeterminate,
    /// Choose one option.
    RadioButton,
    /// Arrange rows and columns.
    Grid,
    /// Adjust a value or configuration.
    Options,
    /// A side pane with content.
    PanelLeft,
    /// Switch between tabbed pages.
    Tab,
    /// Contextual help.
    TooltipQuote,
    /// Layered surfaces and materials.
    Layer,
    /// Navigate to a link.
    Link,
    /// Repeat an action.
    ArrowRepeatAll,
    /// Text and typography.
    TextFont,
    /// A collection of symbols or features.
    Apps,
    /// Add a tab or item.
    Add,
    /// Close a tab or dismiss a surface.
    Dismiss,
    /// Scroll toward the leading edge in left-to-right layout.
    ChevronLeft,
    /// Scroll toward the trailing edge in left-to-right layout.
    ChevronRight,
    /// Expand a child list or open a menu.
    ChevronDown,
    /// Upward disclosure or scroll affordance.
    ChevronUp,
    /// Toggle the navigation pane.
    Navigation,
    /// Opens the navigation search presenter.
    Search,
    /// Navigate back.
    Back,
    /// Open settings.
    Settings,
    /// Open the overflow menu.
    More,
}

impl FluentSymbol {
    /// The glyph from the bundled revision's `codepoints.json`.
    pub fn glyph(self) -> char {
        char::from_u32(match self {
            Self::CursorClick => 58437,
            Self::ToggleLeft => 60816,
            Self::CheckboxChecked => 62093,
            Self::CheckboxIndeterminate => 58110,
            Self::RadioButton => 63044,
            Self::Grid => 62562,
            Self::Options => 62855,
            Self::PanelLeft => 59568,
            Self::Tab => 63300,
            Self::TooltipQuote => 63419,
            Self::Layer => 62667,
            Self::Link => 62692,
            Self::ArrowRepeatAll => 61809,
            Self::TextFont => 63460,
            Self::Apps => 61747,
            Self::Add => 61704,
            Self::Dismiss => 62312,
            Self::ChevronLeft => 62121,
            Self::ChevronRight => 62127,
            Self::ChevronDown => 62114,
            Self::ChevronUp => 62134,
            Self::Search => 63119,
            Self::Navigation => 62816,
            Self::Back => 61787,
            Self::Settings => 63145,
            Self::More => 59428,
        })
        .unwrap()
    }
}

/// A centred font symbol with inherited foreground and optional RTL mirroring.
/// Presentation follows FontIcon; the glyph shapes use the documented open font substitute.
#[derive(Clone, Debug)]
pub struct FluentIcon {
    /// The symbol to display.
    pub symbol: FluentSymbol,
    /// Glyph size; WinUI's `g_ClientCoreFontSize` is 20.
    pub font_size: f64,
    /// Explicit foreground, or the enclosing control's text foreground.
    pub foreground: Option<Color>,
    /// Whether the glyph is reflected horizontally in right-to-left layout.
    pub mirrored_when_right_to_left: bool,
    /// Identity retained across parent rebuilds.
    pub key: Option<KeyRef>,
}

impl FluentIcon {
    /// Creates a symbol at the source default size.
    pub fn new(symbol: FluentSymbol) -> Self {
        Self {
            symbol,
            font_size: 20.0,
            foreground: None,
            mirrored_when_right_to_left: false,
            key: None,
        }
    }

    /// Sets the glyph size independently of inherited label typography.
    pub fn font_size(mut self, value: f64) -> Self {
        self.font_size = value;
        self
    }

    /// Sets an explicit foreground instead of inheriting the label colour.
    pub fn foreground(mut self, value: Color) -> Self {
        self.foreground = Some(value);
        self
    }

    /// Sets the source `MirroredWhenRightToLeft` property.
    pub fn mirrored_when_right_to_left(mut self, value: bool) -> Self {
        self.mirrored_when_right_to_left = value;
        self
    }

    /// Sets the identity retained across rebuilds.
    pub fn key(mut self, value: KeyRef) -> Self {
        self.key = Some(value);
        self
    }
}

impl StatelessWidget for FluentIcon {
    fn key(&self) -> Option<&KeyRef> {
        self.key.as_ref()
    }

    fn build(&self, app: &mut App, context: BuildContext) -> WidgetRef {
        let mut style = TextStyle::new()
            .font_family(FLUENT_ICON_FONT_FAMILY)
            .font_size(self.font_size)
            .font_weight(FontWeight::NORMAL);
        if let Some(color) = self.foreground {
            style = style.color(color);
        }
        // FontIcon's child TextBlock is centred and disables text scale factors.
        let glyph = Text::new(self.symbol.glyph().to_string())
            .style(style)
            .text_align(TextAlign::Center)
            .text_scaler(TextScaler::NO_SCALING);
        let glyph = Center::new()
            .width_factor(1.0)
            .height_factor(1.0)
            .child(glyph);
        if self.mirrored_when_right_to_left
            && Directionality::of(app, context) == TextDirection::Rtl
        {
            Transform::scale(None, Some(-1.0), Some(1.0))
                .child(glyph)
                .into_widget()
        } else {
            glyph.into_widget()
        }
    }
}
