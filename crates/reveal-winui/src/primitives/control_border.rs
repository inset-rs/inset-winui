//! The background, border and corner radius every template's root `ContentPresenter`, `Border` or `Grid` carries: XAML `Background`, `BorderBrush`, `BorderThickness`, `CornerRadius`, `BackgroundSizing` and `Padding`.

use crate::Brush;
use reveal_embedder::{Canvas, Offset, RRect, Radius, Size};
use reveal_foundation::App;
use reveal_painting::{EdgeInsetsGeometry, draw_rrect};
use reveal_rendering::CustomPainter;
use reveal_widgets::*;

/// XAML `BackgroundSizing`: whether the background stops at the border's inner edge or extends under it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BackgroundSizing {
    #[default]
    InnerBorderEdge,
    OuterBorderEdge,
}

/// A rounded rectangle with a background brush and a border brush, the child inside the border and padding.
#[derive(Clone, Debug)]
pub struct ControlBorder {
    pub child: Option<WidgetRef>,
    pub background: Brush,
    pub border_brush: Brush,
    pub border_thickness: f64,
    pub corner_radius: f64,
    pub background_sizing: BackgroundSizing,
    /// Left, top, right, bottom, inside the border.
    pub padding: [f64; 4],
}

impl ControlBorder {
    pub fn new(background: Brush, border_brush: Brush) -> ControlBorder {
        ControlBorder {
            child: None,
            background,
            border_brush,
            border_thickness: 1.0,
            corner_radius: 0.0,
            background_sizing: BackgroundSizing::InnerBorderEdge,
            padding: [0.0; 4],
        }
    }

    pub fn child<K>(mut self, child: impl IntoWidget<K>) -> ControlBorder {
        self.child = Some(child.into_widget());
        self
    }

    pub fn border_thickness(mut self, thickness: f64) -> ControlBorder {
        self.border_thickness = thickness;
        self
    }

    pub fn corner_radius(mut self, radius: f64) -> ControlBorder {
        self.corner_radius = radius;
        self
    }

    pub fn background_sizing(mut self, sizing: BackgroundSizing) -> ControlBorder {
        self.background_sizing = sizing;
        self
    }

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
            left + thickness,
            top + thickness,
            right + thickness,
            bottom + thickness,
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

/// Paints the border as a stroke centred on the inset outline and the background inside or under it.
#[derive(Clone, Debug, PartialEq)]
struct ControlBorderPainter {
    background: Brush,
    border_brush: Brush,
    border_thickness: f64,
    corner_radius: f64,
    background_sizing: BackgroundSizing,
}

impl CustomPainter for ControlBorderPainter {
    fn paint(&self, _app: &mut App, canvas: &mut Canvas, size: Size) {
        let bounds = Offset::ZERO & size;
        let outer = RRect::from_rect_and_radius(bounds, Radius::circular(self.corner_radius));
        let background_rect = match self.background_sizing {
            BackgroundSizing::InnerBorderEdge => outer.deflate(self.border_thickness),
            BackgroundSizing::OuterBorderEdge => outer,
        };
        if self.background.representative_color().a > 0.0 {
            draw_rrect(canvas, background_rect, &self.background.fill(bounds));
        }
        if self.border_thickness > 0.0 && self.border_brush.representative_color().a > 0.0 {
            let stroke = outer.deflate(self.border_thickness / 2.0);
            draw_rrect(
                canvas,
                stroke,
                &self.border_brush.stroke(bounds, self.border_thickness),
            );
        }
    }

    fn should_repaint(&self, _app: &App, old_delegate: &dyn CustomPainter) -> bool {
        old_delegate.as_any().downcast_ref::<Self>() != Some(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
