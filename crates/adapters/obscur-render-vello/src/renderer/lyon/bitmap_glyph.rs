//! Decode font strikes into image data and place them relative to the baseline.

use std::io::Cursor;

use obscur_geometry::{Point, Rect, Size};
use obscur_scene::color::AlphaMode;
use obscur_scene::paint::SceneImage;
use skrifa::bitmap::{BitmapData, BitmapFormat, BitmapGlyph, Origin};

use crate::conversion::FAUX_ITALIC_SKEW;

#[derive(Clone)]
pub(super) struct Bitmap {
    pub(super) image: SceneImage,
    pub(super) rect: Rect,
    pub(super) shear: f32,
}

impl Bitmap {
    #[allow(clippy::cast_precision_loss)]
    pub(super) fn decode(
        glyph: BitmapGlyph<'_>,
        format: Option<BitmapFormat>,
        units_per_em: u16,
        size: f32,
        oblique: bool,
    ) -> Result<Self, &'static str> {
        let width = glyph.width;
        let height = glyph.height;
        // A broken font must not make one glyph allocate an unbounded buffer.
        let bytes = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .filter(|bytes| *bytes <= 16 * 1024 * 1024)
            .ok_or("bitmap glyph is too large")?;
        let (pixels, alpha) = match glyph.data {
            BitmapData::Bgra(data) if data.len() != bytes => return Err("invalid bitmap glyph"),
            BitmapData::Bgra(data) => (
                data.chunks_exact(4)
                    .flat_map(|p| [p[2], p[1], p[0], p[3]])
                    .collect(),
                AlphaMode::Premultiplied,
            ),
            BitmapData::Png(data) => (decode_png(data, width, height)?, AlphaMode::Straight),
            BitmapData::Mask(mask) => (
                mask.decode(width, height)
                    .map_err(|_| "invalid bitmap glyph mask")?
                    .into_iter()
                    .flat_map(|alpha| [255, 255, 255, alpha])
                    .collect(),
                AlphaMode::Straight,
            ),
        };
        let image = SceneImage::new(width, height, pixels, alpha).ok_or("invalid bitmap glyph")?;
        if units_per_em == 0 || !glyph.ppem_y.is_finite() || glyph.ppem_y <= 0.0 {
            return Err("invalid bitmap glyph scale");
        }
        let scale = size / glyph.ppem_y;
        let units = size / f32::from(units_per_em);
        let bearing_y = if glyph.bearing_y == 0.0 && format == Some(BitmapFormat::Sbix) {
            100.0
        } else {
            glyph.bearing_y
        };
        let bottom_offset = match glyph.placement_origin {
            Origin::TopLeft => 0.0,
            Origin::BottomLeft => height as f32,
        };
        let rect = Rect::new(
            Point::new(
                -glyph.bearing_x * units - glyph.inner_bearing_x * scale,
                bearing_y * units - (glyph.inner_bearing_y + bottom_offset) * scale,
            ),
            Size::new(width as f32 * scale, height as f32 * scale),
        );
        Ok(Self {
            image,
            rect,
            shear: if oblique {
                FAUX_ITALIC_SKEW as f32
            } else {
                0.0
            },
        })
    }
}

fn decode_png(data: &[u8], width: u32, height: u32) -> Result<Vec<u8>, &'static str> {
    let mut decoder = png::Decoder::new(Cursor::new(data));
    decoder.set_transformations(png::Transformations::ALPHA | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|_| "invalid bitmap glyph PNG")?;
    if reader.info().size() != (width, height) {
        return Err("invalid bitmap glyph PNG dimensions");
    }
    let mut pixels = vec![
        0;
        reader
            .output_buffer_size()
            .ok_or("bitmap glyph PNG is too large")?
    ];
    let info = reader
        .next_frame(&mut pixels)
        .map_err(|_| "invalid bitmap glyph PNG pixels")?;
    pixels.truncate(info.buffer_size());
    match info.color_type {
        png::ColorType::Rgba => Ok(pixels),
        png::ColorType::GrayscaleAlpha => Ok(pixels
            .chunks_exact(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect()),
        _ => Err("unsupported bitmap glyph PNG format"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skrifa::bitmap::MaskData;

    fn strike(data: BitmapData<'_>) -> BitmapGlyph<'_> {
        BitmapGlyph {
            data,
            bearing_x: 0.0,
            bearing_y: 0.0,
            inner_bearing_x: 0.0,
            inner_bearing_y: 0.0,
            ppem_x: 12.0,
            ppem_y: 12.0,
            advance: None,
            width: 2,
            height: 2,
            placement_origin: Origin::TopLeft,
        }
    }

    #[test]
    fn preserves_premultiplied_bgra_and_reorders_channels() {
        let bgra = [3, 22, 57, 128].repeat(4);
        let bitmap = Bitmap::decode(strike(BitmapData::Bgra(&bgra)), None, 1000, 12.0, false)
            .expect("BGRA glyph");
        assert_eq!(bitmap.image.alpha_mode(), AlphaMode::Premultiplied);
        assert_eq!(bitmap.image.pixels(), [57, 22, 3, 128].repeat(4));
    }

    #[test]
    fn decodes_padded_mask_rows_and_scales_alpha() {
        let glyph = strike(BitmapData::Mask(MaskData {
            bpp: 2,
            is_packed: false,
            data: &[0b0001_1111, 0b1011_1111],
        }));
        let bitmap = Bitmap::decode(glyph, None, 1000, 12.0, false).expect("mask glyph");
        assert_eq!(bitmap.image.alpha_mode(), AlphaMode::Straight);
        let alpha: Vec<_> = bitmap
            .image
            .pixels()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect();
        assert_eq!(alpha, [0, 85, 170, 255]);
    }
}
