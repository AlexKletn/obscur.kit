//! Find a conservative opaque rectangle without reading back window pixels.

use crate::geometry::{AffineTransform, Point, Rect, Size};
use crate::scene::clip::{Clip, ClipShape};
use crate::scene::command::{DisplayList, PaintCommand};
use crate::scene::paint::{Brush, Paint};
use crate::scene::path::{Path, PathSegment};

impl super::AppRunner {
    pub(super) fn stage_window_occlusion(&mut self, snapshot: &crate::scene::scene::SceneSnapshot) {
        if let Some(blur) = self.blur_request.as_mut() {
            let viewport = self.runtime.viewport();
            let radii = self.runtime.viewport_corner_radii().corners();
            static FULL_BLUR: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
            let opaque = (!*FULL_BLUR.get_or_init(|| {
                std::env::var_os("OBSCUR_WAYLAND_FULL_BLUR").is_some_and(|value| value == "1")
            }))
            .then(|| {
                opaque_rect(
                    snapshot.display_list(),
                    viewport,
                    self.runtime.device_scale(),
                )
            })
            .flatten();
            blur.set_surface_occlusion(
                [viewport.width, viewport.height],
                radii.map(|corner| [corner.x, corner.y]),
                opaque,
            );
        }
    }
}

fn opaque_rect(list: &DisplayList, viewport: Size, scale: f32) -> Option<[i32; 4]> {
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let mut transform = AffineTransform::scale(scale.recip(), scale.recip());
    let mut clip = Some(Rect::new(Point::new(0.0, 0.0), viewport));
    let mut stack = Vec::new();
    let mut best = None;
    let mut area = 0;
    for command in list.iter() {
        match command {
            PaintCommand::PushTransform(next) => {
                stack.push((transform, clip));
                transform = transform.multiply(*next);
            }
            PaintCommand::PushClip(next) => {
                stack.push((transform, clip));
                clip = clip.and_then(|clip| intersect(clip_interior(next, transform)?, clip));
            }
            PaintCommand::PushLayer(layer) => {
                // Replacement can destroy previously proven opacity.
                if !layer.composition().is_src_over() {
                    return None;
                }
                stack.push((transform, clip));
                transform = transform.multiply(layer.transform());
                if layer.opacity() != 1.0
                    || !layer.blend_mode().is_normal()
                    || !layer.effects().is_empty()
                    || layer.clip().is_some()
                {
                    clip = None;
                }
            }
            PaintCommand::PopTransform | PaintCommand::PopClip | PaintCommand::PopLayer => {
                (transform, clip) = stack.pop()?;
            }
            PaintCommand::BackdropFilter { .. } => return None,
            PaintCommand::DrawPath {
                path,
                paint: Paint::Fill(Brush::Solid(color)),
            } if color.color().a == 1.0 => {
                if let Some(clip) = clip
                    && let Some(rect) = path_rect(path)
                        .and_then(|rect| axis_aligned(rect, transform))
                        .and_then(|rect| intersect(rect, clip))
                        .and_then(inward_pixels)
                {
                    let candidate = i64::from(rect[2]) * i64::from(rect[3]);
                    if candidate > area {
                        best = Some(rect);
                        area = candidate;
                    }
                }
            }
            _ => {}
        }
    }
    if stack.is_empty() { best } else { None }
}

fn path_rect(path: &Path) -> Option<Rect> {
    use PathSegment::{Close, LineTo, MoveTo};
    let [MoveTo(a), LineTo(b), LineTo(c), LineTo(d), Close] = path.segments() else {
        return None;
    };
    (a.y == b.y && b.x == c.x && c.y == d.y && d.x == a.x && c.x > a.x && c.y > a.y)
        .then(|| Rect::new(*a, Size::new(c.x - a.x, c.y - a.y)))
}

fn clip_interior(clip: &Clip, transform: AffineTransform) -> Option<Rect> {
    let rect = match clip {
        Clip::Shape(ClipShape::Rect(rect)) => *rect,
        Clip::Shape(ClipShape::RoundedRect { rect, radii }) => {
            let radii = radii.normalized(*rect);
            let top = radii.top_left.y.max(radii.top_right.y);
            let bottom = radii.bottom_left.y.max(radii.bottom_right.y);
            Rect::new(
                Point::new(rect.origin.x, rect.origin.y + top),
                Size::new(rect.size.width, rect.size.height - top - bottom),
            )
        }
        Clip::Shape(ClipShape::Path(path)) => path_rect(path)?,
        _ => return None,
    };
    axis_aligned(rect, transform)
}

fn axis_aligned(rect: Rect, transform: AffineTransform) -> Option<Rect> {
    let [sx, skew_y, skew_x, sy, tx, ty] = transform.coeffs;
    if skew_x != 0.0
        || skew_y != 0.0
        || sx <= 0.0
        || sy <= 0.0
        || rect.size.width <= 0.0
        || rect.size.height <= 0.0
    {
        return None;
    }
    let result = Rect::new(
        Point::new(rect.origin.x * sx + tx, rect.origin.y * sy + ty),
        Size::new(rect.size.width * sx, rect.size.height * sy),
    );
    [
        result.origin.x,
        result.origin.y,
        result.right(),
        result.bottom(),
    ]
    .into_iter()
    .all(f32::is_finite)
    .then_some(result)
}

fn intersect(a: Rect, b: Rect) -> Option<Rect> {
    let x = a.origin.x.max(b.origin.x);
    let y = a.origin.y.max(b.origin.y);
    let width = a.right().min(b.right()) - x;
    let height = a.bottom().min(b.bottom()) - y;
    (width > 0.0 && height > 0.0).then(|| Rect::new(Point::new(x, y), Size::new(width, height)))
}

#[allow(clippy::cast_possible_truncation)]
fn inward_pixels(rect: Rect) -> Option<[i32; 4]> {
    let [x, y, right, bottom] = [rect.origin.x, rect.origin.y, rect.right(), rect.bottom()];
    if ![x, y, right, bottom].into_iter().all(f32::is_finite) {
        return None;
    }
    let (x, y) = (x.ceil() as i32, y.ceil() as i32);
    let (right, bottom) = (right.floor() as i32, bottom.floor() as i32);
    (right > x && bottom > y).then(|| [x, y, right - x, bottom - y])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::CornerRadii;
    use crate::scene::color::Color;
    use crate::scene::layer::{CompositionMode, SceneLayer};

    fn fill(rect: Rect, color: Color) -> PaintCommand {
        PaintCommand::DrawPath {
            path: Path::rect(rect),
            paint: Paint::solid_fill(color),
        }
    }

    #[test]
    fn scaled_rounded_window_marks_only_the_opaque_content_interior() {
        let viewport = Size::new(780.0, 580.0);
        let window = Rect::new(Point::new(0.0, 0.0), viewport);
        let content = Rect::new(Point::new(248.0, 0.0), Size::new(532.0, 580.0));
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushTransform(AffineTransform::scale(2.0, 2.0)),
            fill(window, Color::rgba(0.0, 0.0, 0.0, 0.92)),
            PaintCommand::PushClip(Clip::rounded_rect(window, CornerRadii::uniform(16.0))),
            fill(content, Color::BLACK),
            PaintCommand::PopClip,
            PaintCommand::PopTransform,
        ]);
        assert_eq!(opaque_rect(&list, viewport, 2.0), Some([248, 16, 532, 548]));
    }

    #[test]
    fn translucent_layers_and_replacement_do_not_claim_opaque_pixels() {
        let viewport = Size::new(64.0, 64.0);
        let window = Rect::new(Point::new(0.0, 0.0), viewport);
        let transparent = DisplayList::from_commands(vec![
            PaintCommand::PushLayer(SceneLayer::new().with_opacity(0.92)),
            fill(window, Color::BLACK),
            PaintCommand::PopLayer,
        ]);
        assert_eq!(opaque_rect(&transparent, viewport, 1.0), None);

        let replaced = DisplayList::from_commands(vec![
            fill(window, Color::BLACK),
            PaintCommand::PushLayer(SceneLayer::new().with_composition(CompositionMode::Src)),
            fill(window, Color::TRANSPARENT),
            PaintCommand::PopLayer,
        ]);
        assert_eq!(opaque_rect(&replaced, viewport, 1.0), None);
    }
}
