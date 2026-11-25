//! One `Blob` per image, for as long as the image keeps being drawn: `ImageBlobs`.

mod cached_mask;

use cached_mask::CachedMask;

use std::cell::{Cell, RefCell};

use obscur_identity::FastMap;
use obscur_scene::color::AlphaMode;
use obscur_scene::paint::SceneImage;
use vello::peniko::{Blob, ImageAlphaType, ImageData, ImageFormat};

/// How many frames an unused entry survives before it is dropped.
const KEEP_GENERATIONS: u64 = 2;

// Dimensions are part of the image interpretation even when pixels are shared.
type ImageKey = (usize, u32, u32, u8);

#[doc = include_str!("../../docs/api/conversion/image_blobs.doc/ImageBlobs.md")]
#[derive(Debug, Default, Clone)]
pub(super) struct ImageBlobs {
    /// Uploads keyed by pixel allocation, image dimensions and alpha representation.
    /// Each entry records the last generation in which it was used.
    entries: RefCell<FastMap<ImageKey, (ImageData, u64)>>,
    /// The frame counter the eviction window is measured in.
    generation: Cell<u64>,
    luminance_masks: RefCell<FastMap<ImageKey, CachedMask>>,
}

impl ImageBlobs {
    /// Starts a frame: ages the cache and drops what has not been drawn recently.
    pub(super) fn begin_frame(&self) {
        let now = self.generation.get().wrapping_add(1);
        self.generation.set(now);
        self.entries
            .borrow_mut()
            .retain(|_, (_, seen)| now.saturating_sub(*seen) <= KEEP_GENERATIONS);
        self.luminance_masks
            .borrow_mut()
            .retain(|_, mask| now.saturating_sub(mask.seen) <= KEEP_GENERATIONS);
    }

    pub(super) fn luminance_mask(&self, image: &SceneImage) -> Option<SceneImage> {
        let key = image_key(image, image.alpha_mode().is_premultiplied() as u8);
        let now = self.generation.get();
        let mut masks = self.luminance_masks.borrow_mut();
        if let Some(mask) = masks.get_mut(&key) {
            mask.seen = now;
            return Some(mask.image.clone());
        }
        let mask = luminance_mask(image)?;
        masks.insert(key, CachedMask::new(image, mask.clone(), now));
        Some(mask)
    }

    #[doc = include_str!("../../docs/api/conversion/image_blobs.doc/data_for.md")]
    pub(super) fn data_for(&self, image: &SceneImage, alpha_type: ImageAlphaType) -> ImageData {
        // The allocation itself is the identity. It cannot be recycled under us
        // while this cache holds it, because the entry owns a `Blob` that owns
        // the same `Arc` — so an address that matches is the same pixels.
        let key = image_key(image, alpha_type as u8);
        let now = self.generation.get();
        let mut entries = self.entries.borrow_mut();
        if let Some((data, seen)) = entries.get_mut(&key) {
            *seen = now;
            return data.clone();
        }
        let data = ImageData {
            // Zero-copy: `SharedPixels` is `Sized + AsRef<[u8]>`, so it coerces
            // to the `Arc<dyn AsRef<[u8]>>` a `Blob` holds — only a refcount
            // bump, never a pixel copy.
            data: Blob::new(std::sync::Arc::new(image.shared_pixels())),
            format: ImageFormat::Rgba8,
            alpha_type,
            width: image.width(),
            height: image.height(),
        };
        entries.insert(key, (data.clone(), now));
        data
    }
}

fn image_key(image: &SceneImage, alpha: u8) -> ImageKey {
    (
        image.pixels().as_ptr() as usize,
        image.width(),
        image.height(),
        alpha,
    )
}

/// Converts luminance to alpha before Vello interpolates mask samples.
fn luminance_mask(image: &SceneImage) -> Option<SceneImage> {
    use obscur_scene::color::convert::srgb_decode;

    let mut pixels = Vec::with_capacity(image.pixels().len());
    for pixel in image.pixels().chunks_exact(4) {
        let alpha = f64::from(pixel[3]);
        let coverage = if alpha == 0.0 {
            0
        } else {
            let divisor = match image.alpha_mode() {
                AlphaMode::Straight => 255.0,
                AlphaMode::Premultiplied => alpha,
            };
            let luminance: f64 = pixel[..3]
                .iter()
                .zip([0.2126, 0.7152, 0.0722])
                .map(|(&channel, weight)| {
                    weight * srgb_decode((f64::from(channel) / divisor).min(1.0))
                })
                .sum();
            (luminance * alpha).round() as u8
        };
        pixels.extend_from_slice(&[coverage; 4]);
    }
    SceneImage::new(
        image.width(),
        image.height(),
        pixels,
        AlphaMode::Premultiplied,
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use obscur_scene::color::AlphaMode;

    use super::*;

    fn image() -> SceneImage {
        SceneImage::new(
            2,
            2,
            Arc::<[u8]>::from(vec![0u8; 16]),
            AlphaMode::Premultiplied,
        )
        .expect("a 2x2 rgba image")
    }

    #[test]
    fn a_reused_luminance_mask_keeps_its_atlas_identity_until_eviction() {
        let blobs = ImageBlobs::default();
        let source = image();
        let upload = || {
            let mask = blobs.luminance_mask(&source).unwrap();
            blobs
                .data_for(&mask, ImageAlphaType::AlphaPremultiplied)
                .data
                .id()
        };
        blobs.begin_frame();
        let id = upload();
        blobs.begin_frame();
        assert_eq!(upload(), id);
        for _ in 0..=KEEP_GENERATIONS {
            blobs.begin_frame();
        }
        assert!(blobs.luminance_masks.borrow().is_empty());
        assert_ne!(upload(), id);
    }

    #[test]
    fn shared_pixels_can_have_distinct_image_dimensions() {
        let blobs = ImageBlobs::default();
        let square = image();
        let strip =
            SceneImage::new(4, 1, square.shared_pixels(), AlphaMode::Premultiplied).unwrap();
        let square_data = blobs.data_for(&square, ImageAlphaType::AlphaPremultiplied);
        let strip_data = blobs.data_for(&strip, ImageAlphaType::AlphaPremultiplied);
        assert_eq!((square_data.width, square_data.height), (2, 2));
        assert_eq!((strip_data.width, strip_data.height), (4, 1));
        assert_ne!(square_data.data.id(), strip_data.data.id());
    }

    #[test]
    fn the_same_pixels_keep_the_same_blob_across_frames() {
        // The whole point. vello's atlas is keyed by the blob's id, so a new id
        // is a guaranteed miss and a full re-upload of the pixels — every frame,
        // for every image on screen.
        let blobs = ImageBlobs::default();
        let image = image();

        blobs.begin_frame();
        let first = blobs.data_for(&image, ImageAlphaType::AlphaPremultiplied);
        blobs.begin_frame();
        let second = blobs.data_for(&image, ImageAlphaType::AlphaPremultiplied);

        assert_eq!(
            first.data.id(),
            second.data.id(),
            "the pixels did not change, so the upload must not be repeated"
        );
    }

    #[test]
    fn an_image_nobody_draws_stops_being_held() {
        // The other half: the cache owns an `Arc` on the pixels, so never
        // evicting would be a leak that grows with every image ever shown.
        let blobs = ImageBlobs::default();
        let image = image();

        blobs.begin_frame();
        let _ = blobs.data_for(&image, ImageAlphaType::AlphaPremultiplied);
        for _ in 0..=KEEP_GENERATIONS + 1 {
            blobs.begin_frame();
        }
        assert!(
            blobs.entries.borrow().is_empty(),
            "an image off screen for longer than the eviction window is dropped"
        );
    }

    #[test]
    fn the_same_pixels_under_a_different_alpha_mode_are_a_different_upload() {
        // Alpha mode travels in the `ImageData`, not in the pixels, so a cache
        // keyed on the buffer alone would hand back an image vello would blend
        // wrongly.
        let blobs = ImageBlobs::default();
        let image = image();

        blobs.begin_frame();
        let premultiplied = blobs.data_for(&image, ImageAlphaType::AlphaPremultiplied);
        let straight = blobs.data_for(&image, ImageAlphaType::Alpha);

        assert_eq!(premultiplied.alpha_type, ImageAlphaType::AlphaPremultiplied);
        assert_eq!(straight.alpha_type, ImageAlphaType::Alpha);
    }
}
