#![doc = include_str!("image_painter.doc/_module.md")]

use obscur_scene::color::AlphaMode;
use obscur_scene::paint::SceneImage;
use vello::Scene;
use vello::kurbo::{Affine, Rect as KurboRect};
use vello::peniko::{self, Blob, ImageAlphaType, ImageData, ImageFormat};

#[doc = include_str!("image_painter.doc/MAX_TILES_PER_AXIS.md")]
const MAX_TILES_PER_AXIS: i32 = 8192;

#[doc = include_str!("image_painter.doc/MIN_TILE_EXTENT.md")]
const MIN_TILE_EXTENT: f32 = 0.5;

#[doc = include_str!("image_painter.doc/draw_image.md")]
#[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
pub(super) fn draw_image(
    scene: &mut Scene,
    image: &SceneImage,
    tile: &obscur_geometry::Rect,
    region: &obscur_geometry::Rect,
    repeat_x: bool,
    repeat_y: bool,
    transform: Affine,
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

    let alpha_type = match image.alpha_mode() {
        AlphaMode::Straight => ImageAlphaType::Alpha,
        AlphaMode::Premultiplied => ImageAlphaType::AlphaPremultiplied,
    };
    let data = ImageData {
        // Zero-copy: `SharedPixels` is `Sized + AsRef<[u8]>`, so it coerces to
        // the `Arc<dyn AsRef<[u8]>>` a `Blob` holds — only a refcount bump,
        // never a pixel copy, whichever shape the bytes arrived in.
        data: Blob::new(std::sync::Arc::new(image.shared_pixels())),
        format: ImageFormat::Rgba8,
        alpha_type,
        width: iw,
        height: ih,
    };

    // Confine drawing to `region` unless a single tile already sits inside it.
    let contained = !repeat_x
        && !repeat_y
        && tile.origin.x >= region.origin.x - 1e-3
        && tile.origin.y >= region.origin.y - 1e-3
        && tile.origin.x + tile.size.width <= region.origin.x + region.size.width + 1e-3
        && tile.origin.y + tile.size.height <= region.origin.y + region.size.height + 1e-3;
    let clipped = !contained;
    if clipped {
        let clip = KurboRect::new(
            f64::from(region.origin.x),
            f64::from(region.origin.y),
            f64::from(region.origin.x + region.size.width),
            f64::from(region.origin.y + region.size.height),
        );
        scene.push_layer(
            peniko::Fill::NonZero,
            peniko::BlendMode::default(),
            1.0,
            transform,
            &clip,
        );
    }

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
    // Intrinsic → tile: scale the natural `iw × ih` to the tile size (constant
    // across copies), then translate to each copy's origin.
    let scale = Affine::scale_non_uniform(
        f64::from(tile.size.width) / f64::from(iw),
        f64::from(tile.size.height) / f64::from(ih),
    );
    for j in j0..=j1 {
        for i in i0..=i1 {
            let ox = f64::from(tile.origin.x) + f64::from(i) * f64::from(tile.size.width);
            let oy = f64::from(tile.origin.y) + f64::from(j) * f64::from(tile.size.height);
            let place = Affine::translate((ox, oy));
            scene.draw_image(&data, transform * place * scale);
        }
    }

    if clipped {
        scene.pop_layer();
    }
}

#[doc = include_str!("image_painter.doc/tile_index_range.md")]
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
    if end - start + 1 > MAX_TILES_PER_AXIS {
        end = start + MAX_TILES_PER_AXIS - 1;
    }
    (start, end)
}
