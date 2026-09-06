//! XAML brushes as the templates use them: solid colours and the two elevation gradients of `Common_themeresources_any.xaml`.

use reveal_embedder::valo::{
    GradientStop, Matrix, Paint, PaintStyle, Point, Shader, SpreadMode, Stroke,
};
use reveal_embedder::{Color, Rect};

/// A XAML `Brush` a template paints with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Brush {
    /// `SolidColorBrush`.
    Solid(Color),
    /// `ControlElevationBorderBrush` and `AccentControlElevationBorderBrush`: an absolute vertical gradient over the bottom three pixels of the element (XAML `MappingMode="Absolute"`, `EndPoint="0,3"`, flipped by `ScaleY="-1"`), so the bottom edge of a border reads darker. Stops are (offset, colour) as recorded.
    ControlElevation([(f64, Color); 2]),
    /// `CircleElevationBorderBrush`: a vertical gradient over the whole element (`MappingMode="RelativeToBoundingBox"`), for round knobs.
    CircleElevation([(f64, Color); 2]),
}

impl Brush {
    /// The colour a solid brush is; a gradient answers its last stop, the colour it settles to.
    pub fn representative_color(self) -> Color {
        match self {
            Brush::Solid(color) => color,
            Brush::ControlElevation(stops) | Brush::CircleElevation(stops) => stops[1].1,
        }
    }

    /// A fill paint for this brush over `bounds`, in the canvas coordinates the shape is drawn in.
    pub fn fill(self, bounds: Rect) -> Paint {
        match self {
            Brush::Solid(color) => Paint::from_color(color.into()),
            Brush::ControlElevation(stops) => Paint::from_shader(Shader::Linear {
                start: Point::new(bounds.left as f32, bounds.bottom as f32),
                end: Point::new(bounds.left as f32, bounds.bottom as f32 - 3.0),
                stops: gradient_stops(stops),
                spread: SpreadMode::Pad,
                local: Matrix::IDENTITY,
            }),
            Brush::CircleElevation(stops) => Paint::from_shader(Shader::Linear {
                start: Point::new(bounds.left as f32, bounds.top as f32),
                end: Point::new(bounds.left as f32, bounds.bottom as f32),
                stops: gradient_stops(stops),
                spread: SpreadMode::Pad,
                local: Matrix::IDENTITY,
            }),
        }
    }

    /// A stroke paint of `width` for this brush over `bounds`.
    pub fn stroke(self, bounds: Rect, width: f64) -> Paint {
        let mut paint = self.fill(bounds);
        paint.style = PaintStyle::Stroke(Stroke::new(width as f32));
        paint
    }
}

fn gradient_stops(stops: [(f64, Color); 2]) -> Vec<GradientStop> {
    stops
        .into_iter()
        .map(|(offset, color)| GradientStop {
            offset: offset as f32,
            color: color.into(),
        })
        .collect()
}
