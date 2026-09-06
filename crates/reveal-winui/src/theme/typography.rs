//! WinUI's type ramp: the `*TextBlockStyle` styles of `TextBlock_themeresources.xaml` (Fluent 2 sizes) as reveal text styles in the bundled Selawik, Microsoft's open metric-compatible stand-in for Segoe UI Variable.

use super::generated::*;
use reveal_embedder::{Color, FontWeight};
use reveal_painting::TextStyle;

/// The family name the gallery registers Selawik under; `XamlAutoFontFamily` resolves to Segoe UI Variable on Windows.
pub const FONT_FAMILY: &str = "Selawik";

/// XAML `ControlContentThemeFontSize`: the size control content (button labels, toggle content) uses.
pub const CONTROL_CONTENT_FONT_SIZE: f64 = 14.0;

/// One of WinUI's text block styles; XAML `{StaticResource BodyTextBlockStyle}` and its siblings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextBlockStyle {
    Caption,
    Body,
    BodyStrong,
    BodyLarge,
    BodyLargeStrong,
    Subtitle,
    Title,
    TitleLarge,
    Display,
}

impl TextBlockStyle {
    /// The style's size and weight from `TextBlock_themeresources.xaml`: every style is based on `BaseTextBlockStyle` (SemiBold) and the body and caption styles set `Normal`.
    pub fn font(self) -> (f64, FontWeight) {
        match self {
            TextBlockStyle::Caption => (CAPTION_TEXT_BLOCK_FONT_SIZE, FontWeight::W400),
            TextBlockStyle::Body => (BODY_TEXT_BLOCK_FONT_SIZE, FontWeight::W400),
            TextBlockStyle::BodyStrong => (BODY_TEXT_BLOCK_FONT_SIZE, FontWeight::W600),
            TextBlockStyle::BodyLarge => (BODY_LARGE_TEXT_BLOCK_FONT_SIZE, FontWeight::W400),
            TextBlockStyle::BodyLargeStrong => (BODY_LARGE_TEXT_BLOCK_FONT_SIZE, FontWeight::W600),
            TextBlockStyle::Subtitle => (SUBTITLE_TEXT_BLOCK_FONT_SIZE, FontWeight::W600),
            TextBlockStyle::Title => (TITLE_TEXT_BLOCK_FONT_SIZE, FontWeight::W600),
            TextBlockStyle::TitleLarge => (TITLE_LARGE_TEXT_BLOCK_FONT_SIZE, FontWeight::W600),
            TextBlockStyle::Display => (DISPLAY_TEXT_BLOCK_FONT_SIZE, FontWeight::W600),
        }
    }

    /// The reveal text style in `color` (a `TextBlock`'s `Foreground`).
    pub fn text_style(self, color: Color) -> TextStyle {
        let (size, weight) = self.font();
        control_text_style(size, weight, color)
    }
}

/// A text style in the control font; XAML `ContentControlThemeFontFamily` at `size` and `weight`.
pub fn control_text_style(size: f64, weight: FontWeight, color: Color) -> TextStyle {
    TextStyle::new()
        .font_family(FONT_FAMILY)
        .font_size(size)
        .font_weight(weight)
        .color(color)
}
