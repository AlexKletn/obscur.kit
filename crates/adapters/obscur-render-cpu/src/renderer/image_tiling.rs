#![doc = include_str!("image_tiling.doc/_module.md")]

use obscur_geometry::Rect;
use obscur_scene::color::AlphaMode;
use obscur_scene::paint::SceneImage;
use obscur_scene::path::Path;

use crate::renderer::clip_mask::build_clip_mask;
use crate::renderer::skia_lowering::to_sk_path;

#[doc = include_str!("image_tiling.doc/MAX_TILES_PER_AXIS.md")]
const MAX_TILES_PER_AXIS: i32 = 8192;

#[doc = include_str!("image_tiling.doc/MIN_TILE_EXTENT.md")]
const MIN_TILE_EXTENT: f32 = 0.5;

#[doc = include_str!("image_tiling.doc/draw_image.md")]
#[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
pub(crate) fn draw_image(
    pixmap: &mut tiny_skia::Pixmap,
    image: &SceneImage,
    tile: Rect,
    region: Rect,
    repeat_x: bool,
    repeat_y: bool,
    clip: Option<&tiny_skia::Mask>,
    transform: tiny_skia::Transform,
) {
    let (iw, ih) = (image.width(), image.height());
    if iw == 0
        || ih == 0
        || tile.size.width <= 0.0
        || tile.size.height <= 0.0
        || region.size.width <= 0.0
        || region.size.height <= 0.0
    {
        return;
    }
    // A tile too small to tile safely repeats as a single copy on that axis.
    let repeat_x = repeat_x && tile.size.width > MIN_TILE_EXTENT;
    let repeat_y = repeat_y && tile.size.height > MIN_TILE_EXTENT;

    // tiny-skia stores premultiplied RGBA: premultiply a straight image, pass a
    // premultiplied one through unchanged, so both backends show the same picture.
    let premultiplied = match image.alpha_mode() {
        AlphaMode::Premultiplied => image.pixels().to_vec(),
        AlphaMode::Straight => premultiply_rgba(image.pixels()),
    };
    let Some(size) = tiny_skia::IntSize::from_wh(iw, ih) else {
        return;
    };
    let Some(src) = tiny_skia::Pixmap::from_vec(premultiplied, size) else {
        return;
    };

    // The single tile fully inside `region` with no tiling needs no extra clip —
    // the exact pre-placement stretch path. Otherwise confine drawing to `region`
    // by intersecting a `region` mask into the active clip.
    let contained = !repeat_x
        && !repeat_y
        && tile.origin.x >= region.origin.x - 1e-3
        && tile.origin.y >= region.origin.y - 1e-3
        && tile.origin.x + tile.size.width <= region.origin.x + region.size.width + 1e-3
        && tile.origin.y + tile.size.height <= region.origin.y + region.size.height + 1e-3;
    let region_mask = if contained {
        None
    } else {
        let Some(region_path) = to_sk_path(&Path::rect(region)) else {
            return;
        };
        build_clip_mask(
            pixmap.width(),
            pixmap.height(),
            &region_path,
            tiny_skia::FillRule::Winding,
            clip,
            transform,
        )
    };
    let effective_clip = if contained {
        clip
    } else {
        region_mask.as_ref()
    };

    // Tile index ranges: `[i0, i1] × [j0, j1]` copies stepping by tile w/h from the
    // tile origin, covering `region`. A non-repeating axis is the single index 0.
    let (i0, i1) = tile_index_range(
        repeat_x,
        region.origin.x,
        region.origin.x + region.size.width,
        tile.origin.x,
        tile.size.width,
    );
    let (j0, j1) = tile_index_range(
        repeat_y,
        region.origin.y,
        region.origin.y + region.size.height,
        tile.origin.y,
        tile.size.height,
    );

    let paint = tiny_skia::PixmapPaint::default();
    let (sx, sy) = (tile.size.width / iw as f32, tile.size.height / ih as f32);
    for j in j0..=j1 {
        for i in i0..=i1 {
            let ox = tile.origin.x + i as f32 * tile.size.width;
            let oy = tile.origin.y + j as f32 * tile.size.height;
            // Intrinsic → tile: scale the natural `iw × ih` to the tile size, then
            // translate to this copy's origin, all under the running transform.
            let place = transform
                .pre_concat(tiny_skia::Transform::from_translate(ox, oy))
                .pre_concat(tiny_skia::Transform::from_scale(sx, sy));
            pixmap.draw_pixmap(0, 0, src.as_ref(), &paint, place, effective_clip);
        }
    }
}

#[doc = include_str!("image_tiling.doc/tile_index_range.md")]
#[allow(clippy::cast_possible_truncation)]
fn tile_index_range(repeat: bool, r0: f32, r1: f32, t0: f32, ts: f32) -> (i32, i32) {
    if !repeat || ts <= 0.0 {
        return (0, 0);
    }
    let start_f = ((r0 - t0) / ts).floor();
    let end_f = ((r1 - t0) / ts).ceil();
    if !start_f.is_finite() || !end_f.is_finite() {
        return (0, 0);
    }
    let start = start_f as i32 - 1;
    let mut end = end_f as i32 + 1;
    // Bound the loop: never draw more than MAX_TILES_PER_AXIS copies on this axis.
    if end - start + 1 > MAX_TILES_PER_AXIS {
        end = start + MAX_TILES_PER_AXIS - 1;
    }
    (start, end)
}

#[doc = include_str!("image_tiling.doc/premultiply_rgba.md")]
#[allow(clippy::cast_possible_truncation)]
fn premultiply_rgba(straight: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(straight.len());
    for px in straight.chunks_exact(4) {
        let a = u16::from(px[3]);
        // Rounded `c * a / 255` per channel (a == 255 is an exact identity).
        // The result is always `<= 255`, so the `as u8` never truncates.
        let mul = |c: u8| -> u8 { ((u16::from(c) * a + 127) / 255) as u8 };
        out.push(mul(px[0]));
        out.push(mul(px[1]));
        out.push(mul(px[2]));
        out.push(px[3]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_non_repeating_axis_is_the_single_copy_at_index_zero() {
        assert_eq!(tile_index_range(false, 0.0, 100.0, 0.0, 10.0), (0, 0));
    }

    #[test]
    fn a_degenerate_tile_extent_cannot_spin_the_loop() {
        assert_eq!(tile_index_range(true, 0.0, 100.0, 0.0, 0.0), (0, 0));
        assert_eq!(tile_index_range(true, 0.0, f32::INFINITY, 0.0, 1.0), (0, 0));
    }

    #[test]
    fn a_repeating_axis_covers_the_region_with_one_tile_of_bleed() {
        // Copies of extent 10 from x = 0 covering [0, 30]: indices 0..=3, padded by
        // one on each side so a partially-visible edge tile is drawn.
        assert_eq!(tile_index_range(true, 0.0, 30.0, 0.0, 10.0), (-1, 4));
    }

    #[test]
    fn a_pathological_tile_count_is_capped() {
        let (start, end) = tile_index_range(true, 0.0, 1.0e9, 0.0, 1.0);
        assert_eq!(end - start + 1, MAX_TILES_PER_AXIS);
    }

    #[test]
    fn straight_alpha_is_scaled_into_tiny_skias_premultiplied_storage() {
        // Opaque pixels are an exact identity; a half-alpha red halves its red.
        let straight = [255u8, 0, 0, 255, 255, 0, 0, 128];
        assert_eq!(
            premultiply_rgba(&straight),
            vec![255, 0, 0, 255, 128, 0, 0, 128]
        );
    }
}
