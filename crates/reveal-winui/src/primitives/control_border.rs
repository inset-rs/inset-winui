//! The background, border and corner radius every template's root `ContentPresenter`, `Border` or `Grid` carries: XAML `Background`, `BorderBrush`, `BorderThickness`, `CornerRadius`, `BackgroundSizing` and `Padding`.

use crate::Brush;
use reveal_embedder::{Canvas, Offset, RRect, Radius, Size};
use reveal_foundation::App;
use reveal_painting::{EdgeInsets, EdgeInsetsGeometry, draw_drrect, draw_rrect};
use reveal_rendering::CustomPainter;
use reveal_widgets::*;

/// XAML `BackgroundSizing`: whether the background stops at the border's inner edge or extends under it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BackgroundSizing {
    /// Paints the background inside the border's inner edge.
    #[default]
    InnerBorderEdge,
    /// Paints the background beneath the border up to its outer edge.
    OuterBorderEdge,
}

/// A rounded rectangle with a background brush and a border brush, the child inside the border and padding.
#[derive(Clone, Debug)]
pub struct ControlBorder {
    /// The content placed inside the border and padding.
    pub child: Option<WidgetRef>,
    /// The brush filling the area selected by `background_sizing`.
    pub background: Brush,
    /// The brush filling the ring between the inner and outer border outlines.
    pub border_brush: Brush,
    /// Left, top, right, bottom.
    pub border_thickness: [f64; 4],
    /// Top-left, top-right, bottom-right, bottom-left.
    pub corner_radius: [f64; 4],
    /// Whether the background extends beneath the border.
    pub background_sizing: BackgroundSizing,
    /// Left, top, right, bottom, inside the border.
    pub padding: [f64; 4],
}

impl ControlBorder {
    /// Creates border chrome with the supplied brushes and a one-pixel border.
    pub fn new(background: Brush, border_brush: Brush) -> ControlBorder {
        ControlBorder {
            child: None,
            background,
            border_brush,
            border_thickness: [1.0; 4],
            corner_radius: [0.0; 4],
            background_sizing: BackgroundSizing::InnerBorderEdge,
            padding: [0.0; 4],
        }
    }

    /// Sets the content inside the border and padding.
    pub fn child<K>(mut self, child: impl IntoWidget<K>) -> ControlBorder {
        self.child = Some(child.into_widget());
        self
    }

    /// Sets a uniform border width on all four sides.
    pub fn border_thickness(mut self, thickness: f64) -> ControlBorder {
        self.border_thickness = [thickness; 4];
        self
    }

    /// XAML `BorderThickness`, in left, top, right, bottom order.
    pub fn border_thickness_ltrb(mut self, thickness: [f64; 4]) -> ControlBorder {
        self.border_thickness = thickness;
        self
    }

    /// Sets a uniform radius on all four corners.
    pub fn corner_radius(mut self, radius: f64) -> ControlBorder {
        self.corner_radius = [radius; 4];
        self
    }

    /// XAML `CornerRadius`, in top-left, top-right, bottom-right, bottom-left order.
    pub fn corner_radius_corners(mut self, radii: [f64; 4]) -> ControlBorder {
        self.corner_radius = radii;
        self
    }

    /// Sets whether the background fills beneath the border.
    pub fn background_sizing(mut self, sizing: BackgroundSizing) -> ControlBorder {
        self.background_sizing = sizing;
        self
    }

    /// Sets additional insets inside the border in left, top, right, bottom order.
    pub fn padding(mut self, padding: [f64; 4]) -> ControlBorder {
        self.padding = padding;
        self
    }
}

impl StatelessWidget for ControlBorder {
    fn build(&self, _app: &mut App, _context: BuildContext) -> WidgetRef {
        let [left, top, right, bottom] = self.padding;
        let thickness = self.border_thickness;
        let painter = ControlBorderPainter {
            background: self.background,
            border_brush: self.border_brush,
            border_thickness: thickness,
            corner_radius: self.corner_radius,
            background_sizing: self.background_sizing,
        };
        let inner = Padding::new(EdgeInsetsGeometry::from_ltrb(
            left + thickness[0],
            top + thickness[1],
            right + thickness[2],
            bottom + thickness[3],
        ));
        let inner = match &self.child {
            Some(child) => inner.child(child.clone()),
            None => inner,
        };
        CustomPaint::new()
            .painter(painter)
            .child(inner)
            .into_widget()
    }
}

/// Paints the border between its outer and inner rounded outlines and the background inside or under it.
#[derive(Clone, Debug, PartialEq)]
struct ControlBorderPainter {
    background: Brush,
    border_brush: Brush,
    border_thickness: [f64; 4],
    corner_radius: [f64; 4],
    background_sizing: BackgroundSizing,
}

impl CustomPainter for ControlBorderPainter {
    fn paint(&self, _app: &mut App, canvas: &mut Canvas, size: Size) {
        let bounds = Offset::ZERO & size;
        let [top_left, top_right, bottom_right, bottom_left] =
            self.corner_radius.map(Radius::circular);
        let outer =
            RRect::from_rect_and_corners(bounds, top_left, top_right, bottom_right, bottom_left);
        let [left, top, right, bottom] = self.border_thickness;
        let inner = EdgeInsets::from_ltrb(left, top, right, bottom).deflate_rrect(outer);
        let background_rect = match self.background_sizing {
            BackgroundSizing::InnerBorderEdge => inner,
            BackgroundSizing::OuterBorderEdge => outer,
        };
        if self.background.representative_color().a > 0.0 {
            draw_rrect(canvas, background_rect, &self.background.fill(bounds));
        }
        if self
            .border_thickness
            .iter()
            .any(|thickness| *thickness > 0.0)
            && self.border_brush.representative_color().a > 0.0
        {
            draw_drrect(canvas, outer, inner, &self.border_brush.fill(bounds));
        }
    }

    fn should_repaint(&self, _app: &App, old_delegate: &dyn CustomPainter) -> bool {
        old_delegate.as_any().downcast_ref::<Self>() != Some(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
