//! ElevationHelper.cpp and DropShadowRecipe.h, using Inset's native shadow blur.

use crate::{OVERLAY_CORNER_RADIUS, Theme};
use inset_embedder::{BlurStyle, Canvas, ClipOp, Color, Offset, RRect, Radius, Size};
use inset_foundation::App;
use inset_painting::{BoxShadow, draw_rrect};
use inset_rendering::CustomPainter;

/// Paints only outside the popup, so acrylic samples an undarkened background.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct PopupShadow {
    /// Theme used by the popup's content.
    pub theme: Theme,

    /// Number of submenu ancestors, as used by ApplyElevationEffect.
    pub depth: usize,
}

impl PopupShadow {
    /// Source shadow colors and offsets, with blur radii interpreted by BoxShadow.
    fn shadows(&self) -> [BoxShadow; 2] {
        // ElevationHelper uses Translation.Z = 32 + depth * 8; the recipe halves it.
        let elevation = (16.0 + self.depth as f64 * 4.0).min(64.0);
        let (ambient_blur, ambient_offset, ambient_opacity, directional_opacity) =
            if elevation <= 16.0 {
                (
                    2.0,
                    0.0,
                    0.0,
                    if self.theme == Theme::Light {
                        0.14
                    } else {
                        0.26
                    },
                )
            } else {
                let (ambient, directional) = if self.theme == Theme::Light {
                    (0.15, 0.19)
                } else {
                    (0.37, 0.37)
                };
                (elevation / 3.0, 2.0, ambient, directional)
            };

        let shadow = |blur, y, opacity: f64| {
            BoxShadow::new(
                Color::from_argb((opacity * 255.0) as i32, 0, 0, 0),
                Offset::new(0.0, y),
                blur,
                0.0,
                BlurStyle::Normal,
            )
        };

        [
            shadow(ambient_blur, ambient_offset, ambient_opacity),
            shadow(elevation, elevation * 0.5, directional_opacity),
        ]
    }
}

impl CustomPainter for PopupShadow {
    fn paint(&self, _app: &mut App, canvas: &mut Canvas, size: Size) {
        let bounds = Offset::ZERO & size;
        let radius = OVERLAY_CORNER_RADIUS[0].min(size.shortest_side() / 2.0);
        let outline = RRect::from_rect_and_radius(bounds, Radius::circular(radius));

        canvas.save();
        canvas.clip_rrect(bounds, radius as f32, ClipOp::Difference);
        for shadow in self.shadows() {
            if shadow.color.a == 0.0 {
                continue;
            }
            draw_rrect(canvas, outline.shift(shadow.offset), &shadow.to_paint());
        }
        canvas.restore();
    }

    /// The shadow never enlarges the popup's interactive area.
    fn hit_test(&self, _app: &App, _position: Offset) -> Option<bool> {
        Some(false)
    }

    fn should_repaint(&self, _app: &App, old_delegate: &dyn CustomPainter) -> bool {
        old_delegate.as_any().downcast_ref::<Self>() != Some(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
