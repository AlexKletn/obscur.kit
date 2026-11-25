use obscur_geometry::{Point, Rect};
use obscur_scene::path::{FillRule, Path};
use obscur_style::property::LengthExpr;

use super::resolve_length;

pub(crate) struct ClipPolygon<'a> {
    points: &'a [[LengthExpr; 2]],
    bounds: Rect,
    even_odd: bool,
}

impl<'a> ClipPolygon<'a> {
    pub(super) fn new(points: &'a [[LengthExpr; 2]], bounds: Rect, even_odd: bool) -> Self {
        Self {
            points,
            bounds,
            even_odd,
        }
    }
}

impl ClipPolygon<'_> {
    fn vertices(&self) -> impl Iterator<Item = Point> + '_ {
        self.points.iter().map(|[x, y]| {
            Point::new(
                self.bounds.origin.x + resolve_length(x, self.bounds.size.width),
                self.bounds.origin.y + resolve_length(y, self.bounds.size.height),
            )
        })
    }

    pub(super) fn path(&self) -> Path {
        let mut path = Path::with_fill_rule(if self.even_odd {
            FillRule::EvenOdd
        } else {
            FillRule::NonZero
        });
        let mut vertices = self.vertices();
        if let Some(first) = vertices.next() {
            path.move_to(first);
            for point in vertices {
                path.line_to(point);
            }
            path.close();
        }
        path
    }

    pub(crate) fn contains(&self, point: Point) -> bool {
        if self.points.len() < 3 {
            return false;
        }
        let mut vertices = self.vertices();
        let Some(first) = vertices.next() else {
            return false;
        };
        let mut start = first;
        let mut winding = 0_i64;
        for end in vertices.chain(std::iter::once(first)) {
            let cross = (f64::from(end.x) - f64::from(start.x))
                * (f64::from(point.y) - f64::from(start.y))
                - (f64::from(end.y) - f64::from(start.y))
                    * (f64::from(point.x) - f64::from(start.x));
            if start != end
                && cross == 0.0
                && (start.x.min(end.x)..=start.x.max(end.x)).contains(&point.x)
                && (start.y.min(end.y)..=start.y.max(end.y)).contains(&point.y)
            {
                return true;
            }
            if start.y <= point.y && end.y > point.y && cross > 0.0 {
                winding += 1;
            } else if start.y > point.y && end.y <= point.y && cross < 0.0 {
                winding -= 1;
            }
            start = end;
        }
        if self.even_odd {
            winding % 2 != 0
        } else {
            winding != 0
        }
    }
}
