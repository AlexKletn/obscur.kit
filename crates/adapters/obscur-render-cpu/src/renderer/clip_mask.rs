#![doc = include_str!("../../docs/api/renderer/clip_mask.doc/_module.md")]

use obscur_scene::clip::{Clip, ClipShape};
use obscur_scene::path::Path;

use crate::renderer::skia_lowering::{to_sk_fill_rule, to_sk_path};

/// Lowers a geometric [`ClipShape`] to a domain [`Path`] this backend can
/// rasterize into a clip mask. Rectangles and rounded rectangles are expanded to
/// their outline; a path shape is used directly.
fn clip_shape_to_domain_path(shape: &ClipShape) -> Path {
    match shape {
        ClipShape::Rect(rect) => Path::rect(*rect),
        ClipShape::RoundedRect { rect, radii } => Path::rounded_rect(*rect, *radii),
        ClipShape::Path(path) => path.clone(),
    }
}

#[doc = include_str!("../../docs/api/renderer/clip_mask.doc/empty_clip_mask.md")]
fn empty_clip_mask(width: u32, height: u32) -> Option<tiny_skia::Mask> {
    tiny_skia::Mask::new(width, height)
}

#[doc = include_str!("../../docs/api/renderer/clip_mask.doc/resolve_clip_entry.md")]
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
        // The renderer applies image masks when it composites isolated groups.
        // This path inherits the enclosing clip for an unavailable coverage mask.
        Clip::Mask(_) | Clip::ImageMask { .. } => parent.cloned(),
    }
}

#[doc = include_str!("../../docs/api/renderer/clip_mask.doc/build_clip_mask.md")]
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

/// Rasterizes image coverage in device space for an isolated group composite.
pub(super) fn image_mask(
    clip: &Clip,
    parent: Option<&tiny_skia::Mask>,
    width: u32,
    height: u32,
    transform: tiny_skia::Transform,
) -> Option<tiny_skia::Mask> {
    use obscur_scene::color::{AlphaMode, convert::srgb_decode};

    let Clip::ImageMask {
        kind,
        image,
        bounds,
    } = clip
    else {
        return None;
    };
    let mut raster = tiny_skia::Pixmap::new(width, height)?;
    if bounds.size.width > 0.0
        && bounds.size.height > 0.0
        && image.width() > 0
        && image.height() > 0
    {
        let mut source = tiny_skia::Pixmap::new(image.width(), image.height())?;
        for (pixel, coverage) in image
            .pixels()
            .chunks_exact(4)
            .zip(source.data_mut().chunks_exact_mut(4))
        {
            let alpha = pixel[3];
            let amount = if kind.kind().is_luminance() && alpha > 0 {
                let divisor = match image.alpha_mode() {
                    AlphaMode::Straight => 255.0,
                    AlphaMode::Premultiplied => f64::from(alpha),
                };
                let luminance: f64 = pixel[..3]
                    .iter()
                    .zip([0.2126, 0.7152, 0.0722])
                    .map(|(&channel, weight)| {
                        weight * srgb_decode((f64::from(channel) / divisor).min(1.0))
                    })
                    .sum();
                (luminance * f64::from(alpha)).round() as u8
            } else {
                alpha
            };
            coverage.fill(amount);
        }
        let placement = transform
            .pre_concat(tiny_skia::Transform::from_translate(
                bounds.origin.x,
                bounds.origin.y,
            ))
            .pre_concat(tiny_skia::Transform::from_scale(
                bounds.size.width / image.width() as f32,
                bounds.size.height / image.height() as f32,
            ));
        let paint = tiny_skia::PixmapPaint {
            quality: tiny_skia::FilterQuality::Bilinear,
            ..Default::default()
        };
        raster.draw_pixmap(0, 0, source.as_ref(), &paint, placement, None);
    }
    let mut mask = tiny_skia::Mask::from_pixmap(raster.as_ref(), tiny_skia::MaskType::Alpha);
    if let Some(parent) = parent {
        for (amount, &limit) in mask.data_mut().iter_mut().zip(parent.data()) {
            *amount = ((u16::from(*amount) * u16::from(limit) + 127) / 255) as u8;
        }
    }
    Some(mask)
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
