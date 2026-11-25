//! Update rectangle corners in a retained mesh without emitting its other draws.

use obscur_geometry::{AffineTransform, Point, Rect, Size};
use obscur_scene::paint::Brush;
use obscur_scene::path::{Path, PathSegment};

use super::Mesh;

impl Mesh {
    pub(super) fn update_rect_fill(
        &mut self,
        command: usize,
        (old, old_brush): (&Path, &Brush),
        (new, new_brush): (&Path, &Brush),
        transform: AffineTransform,
    ) -> bool {
        let (Some(old), Some(new)) = (path_rect(old), path_rect(new)) else {
            return false;
        };
        if !self.update_rect(command, old, new, transform) {
            return false;
        }
        let first = self.rect_vertices[command].start;
        old_brush == new_brush
            || self
                .gradients
                .update_linear(self.vertices[first].gradient, new_brush)
    }

    pub(super) fn update_rect(
        &mut self,
        command: usize,
        old: Rect,
        new: Rect,
        transform: AffineTransform,
    ) -> bool {
        let edges = |rect: Rect| [rect.origin.x, rect.origin.y, rect.right(), rect.bottom()];
        let [left, top, right, bottom] = edges(old);
        let [new_left, new_top, new_right, new_bottom] = edges(new);
        if !edges(old).into_iter().chain(edges(new)).all(f32::is_finite)
            || left >= right
            || top >= bottom
            || new_left >= new_right
            || new_top >= new_bottom
        {
            return false;
        }
        let vertices = &mut self.vertices[self.rect_vertices[command].clone()];
        if vertices.len() != 4
            || !vertices.iter().all(|v| {
                let [x, y] = v.position;
                (x == left || x == right) && (y == top || y == bottom)
            })
        {
            return false;
        }
        for vertex in vertices {
            let [x, y] = vertex.position;
            let point = Point::new(
                if x == left { new_left } else { new_right },
                if y == top { new_top } else { new_bottom },
            );
            let Ok(position) = super::vertex_position(true, transform, point) else {
                return false;
            };
            vertex.position = position;
        }
        true
    }
}

pub(in crate::renderer::lyon) fn path_rect(path: &Path) -> Option<Rect> {
    use PathSegment::{Close, LineTo, MoveTo};
    let [MoveTo(a), LineTo(b), LineTo(c), LineTo(d), Close] = path.segments() else {
        return None;
    };
    if a.y != b.y || b.x != c.x || c.y != d.y || d.x != a.x {
        return None;
    }
    let rect = Rect::new(Point::new(a.x, a.y), Size::new(c.x - a.x, c.y - a.y));
    (rect.right() == c.x && rect.bottom() == c.y).then_some(rect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::SceneConverter;
    use crate::renderer::lyon::geometry::GeometryCache;
    use obscur_scene::clip::Clip;
    use obscur_scene::color::Color;
    use obscur_scene::command::{DisplayList, PaintCommand};
    use obscur_scene::paint::{GradientStop, LinearGradient, Paint};

    fn scene(width: f32, shared_gradient: bool) -> DisplayList {
        let rect = |width| Rect::new(Point::new(0.0, 0.0), Size::new(width, 50.0));
        let gradient = Brush::Linear(LinearGradient::new(
            Point::new(0.0, 0.0),
            Point::new(width, 0.0),
            vec![
                GradientStop::new(0.0, Color::TRANSPARENT),
                GradientStop::new(1.0, Color::WHITE),
            ],
        ));
        let mut commands = vec![
            PaintCommand::PushClip(Clip::rect(rect(width))),
            PaintCommand::DrawPath {
                path: Path::rect(rect(100.0)),
                paint: Paint::solid_fill(Color::BLACK),
            },
            PaintCommand::PopClip,
            PaintCommand::DrawPath {
                path: Path::rect(rect(width)),
                paint: Paint::fill(gradient.clone()),
            },
        ];
        if shared_gradient {
            commands.push(PaintCommand::DrawPath {
                path: Path::rect(rect(100.0)),
                paint: Paint::fill(gradient),
            });
        }
        DisplayList::from_commands(commands)
    }

    #[test]
    fn retains_indices_and_untouched_vertices_for_width_changes() {
        let old = scene(30.0, false);
        let mut mesh = Mesh::default();
        let mut geometry = GeometryCache::default();
        let fonts = SceneConverter::default();
        mesh.lower(&old, (100, 50), &mut geometry, &fonts).unwrap();
        let indices = mesh.indices.clone();
        let middle = bytemuck::cast_slice::<_, u8>(&mesh.vertices[4..8]).to_vec();
        let mut previous = old;
        for width in [80.0, 16.0, 30.0] {
            let next = scene(width, false);
            let mut changed = false;
            assert!(mesh.update_parameters(&previous, &next, &mut changed));
            assert!(changed);
            assert_eq!(mesh.indices, indices);
            assert_eq!(bytemuck::cast_slice::<_, u8>(&mesh.vertices[4..8]), middle);
            previous = next;
        }
        let old = scene(30.0, true);
        mesh.lower(&old, (100, 50), &mut geometry, &fonts).unwrap();
        assert!(!mesh.update_parameters(&old, &scene(80.0, true), &mut false));
    }
}
