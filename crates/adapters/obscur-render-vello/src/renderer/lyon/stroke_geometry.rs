use lyon_tessellation::path::{LineCap, LineJoin};
use obscur_scene::paint::Stroke;
use vello::kurbo;

use crate::conversion::{BrushConverter, ColorConverter};

#[derive(Clone, PartialEq)]
pub(super) enum StrokeGeometry {
    Solid {
        width: f32,
        cap: LineCap,
        join: LineJoin,
        miter_limit: f32,
    },
    Dashed(kurbo::Stroke),
}

impl StrokeGeometry {
    pub(super) fn from_stroke(stroke: &Stroke) -> Result<Self, &'static str> {
        use obscur_scene::paint::{LineCap as Cap, LineJoin as Join};

        if !stroke.width().is_finite() || stroke.width() < 0.0 {
            return Err("invalid stroke width");
        }
        if stroke.is_dashed() {
            return Ok(Self::Dashed(
                BrushConverter::new(ColorConverter::new()).stroke(stroke),
            ));
        }
        Ok(Self::Solid {
            width: stroke.width(),
            cap: match stroke.line_cap() {
                Cap::Butt => LineCap::Butt,
                Cap::Round => LineCap::Round,
                Cap::Square => LineCap::Square,
            },
            join: match stroke.line_join() {
                Join::Miter => LineJoin::Miter,
                Join::Round => LineJoin::Round,
                Join::Bevel => LineJoin::Bevel,
            },
            miter_limit: stroke.miter_limit().max(1.0),
        })
    }
}
