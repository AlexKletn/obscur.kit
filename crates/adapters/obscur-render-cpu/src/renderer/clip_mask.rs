#![doc = include_str!("clip_mask.doc/_module.md")]

use obscur_scene::clip::{Clip, ClipShape};
use obscur_scene::path::Path;

use crate::renderer::skia_lowering::{to_sk_fill_rule, to_sk_path};

#[doc = include_str!("clip_mask.doc/clip_shape_to_domain_path.md")]
fn clip_shape_to_domain_path(shape: &ClipShape) -> Path {
    match shape {
        ClipShape::Rect(rect) => Path::rect(*rect),
        ClipShape::RoundedRect { rect, radii } => Path::rounded_rect(*rect, *radii),
        ClipShape::Path(path) => path.clone(),
    }
}

#[doc = include_str!("clip_mask.doc/empty_clip_mask.md")]
fn empty_clip_mask(width: u32, height: u32) -> Option<tiny_skia::Mask> {
    tiny_skia::Mask::new(width, height)
}

#[doc = include_str!("clip_mask.doc/resolve_clip_entry.md")]
pub(crate) fn resolve_clip_entry(
    clip: &Clip,
    parent: Option<&tiny_skia::Mask>,
    width: u32,
    height: u32,
    transform: tiny_skia::Transform,
) -> Option<tiny_skia::Mask> {
    match clip {
        Clip::Shape(shape) => {
            let domain_path = clip_shape_to_domain_path(shape);
            match to_sk_path(&domain_path) {
                Some(sk_path) => build_clip_mask(
                    width,
                    height,
                    &sk_path,
                    to_sk_fill_rule(domain_path.fill_rule()),
                    parent,
                    transform,
                )
                .or_else(|| parent.cloned()),
                // An empty/degenerate outline encloses no area, so the clip admits
                // nothing — an all-zero mask, NOT the parent. The extent here is
                // known (it is empty); inheriting would paint content the author
                // clipped away, and would disagree with Vello, whose empty outline
                // already admits nothing. Only an allocation failure falls back to
                // the parent.
                None => empty_clip_mask(width, height).or_else(|| parent.cloned()),
            }
        }
        // A mask carries only *how* to read coverage, never the source it comes
        // from, so a mask clip states no extent at all (`Clip::bounds()` is `None`
        // for exactly this reason). Unknown extent is not empty extent: inherit the
        // enclosing clip, which asserts nothing the domain has not stated, and
        // matches Vello. This is the §5.8 fallback, not mask support — coverage is
        // not applied on either backend, and neither raises the diagnostic §5.8
        // asks an explicit fallback to carry.
        Clip::Mask(_) => parent.cloned(),
    }
}

#[doc = include_str!("clip_mask.doc/build_clip_mask.md")]
pub(crate) fn build_clip_mask(
    width: u32,
    height: u32,
    sk_path: &tiny_skia::Path,
    fill_rule: tiny_skia::FillRule,
    parent: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
) -> Option<tiny_skia::Mask> {
    if let Some(parent) = parent {
        let mut mask = parent.clone();
        mask.intersect_path(sk_path, fill_rule, true, transform);
        Some(mask)
    } else {
        let mut mask = tiny_skia::Mask::new(width, height)?;
        mask.fill_path(sk_path, fill_rule, true, transform);
        Some(mask)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Rect, Size};

    fn rect_clip(x: f32, y: f32, w: f32, h: f32) -> Clip {
        Clip::rect(Rect::new(Point::new(x, y), Size::new(w, h)))
    }

    #[test]
    fn a_shape_clip_covers_its_outline_and_nothing_beyond_it() {
        let mask = resolve_clip_entry(
            &rect_clip(2.0, 2.0, 6.0, 6.0),
            None,
            16,
            16,
            tiny_skia::Transform::identity(),
        )
        .expect("mask allocates");
        assert_eq!(mask.data()[5 * 16 + 5], 255, "inside the clip passes");
        assert_eq!(mask.data()[12 * 16 + 12], 0, "outside the clip does not");
    }

    #[test]
    fn a_nested_clip_intersects_its_parent() {
        let identity = tiny_skia::Transform::identity();
        let parent =
            resolve_clip_entry(&rect_clip(0.0, 0.0, 8.0, 16.0), None, 16, 16, identity).expect("m");
        let child = resolve_clip_entry(
            &rect_clip(4.0, 0.0, 12.0, 16.0),
            Some(&parent),
            16,
            16,
            identity,
        )
        .expect("m");
        assert_eq!(child.data()[5 * 16 + 5], 255, "the overlap passes");
        assert_eq!(child.data()[5 * 16 + 1], 0, "outside the parent does not");
        assert_eq!(child.data()[5 * 16 + 10], 0, "outside the child does not");
    }

    #[test]
    fn an_empty_outline_admits_nothing_rather_than_inheriting() {
        let identity = tiny_skia::Transform::identity();
        let parent = resolve_clip_entry(&rect_clip(0.0, 0.0, 16.0, 16.0), None, 16, 16, identity)
            .expect("m");
        let empty = resolve_clip_entry(
            &Clip::Shape(ClipShape::Path(Path::new())),
            Some(&parent),
            16,
            16,
            identity,
        )
        .expect("m");
        assert!(
            empty.data().iter().all(|&c| c == 0),
            "an empty clip outline admits nothing, it does not inherit the parent"
        );
    }
}
