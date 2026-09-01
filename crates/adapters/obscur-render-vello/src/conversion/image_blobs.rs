#![doc = include_str!("image_blobs.doc/_module.md")]

use std::cell::{Cell, RefCell};

use obscur_identity::FastMap;
use obscur_scene::paint::SceneImage;
use vello::peniko::{Blob, ImageAlphaType, ImageData, ImageFormat};

#[doc = include_str!("image_blobs.doc/KEEP_GENERATIONS.md")]
const KEEP_GENERATIONS: u64 = 2;

#[doc = include_str!("image_blobs.doc/ImageBlobs.md")]
#[derive(Debug, Default, Clone)]
pub(super) struct ImageBlobs {
    #[doc = include_str!("image_blobs.doc/entries.md")]
    entries: RefCell<FastMap<(usize, usize), (ImageData, u64)>>,
    #[doc = include_str!("image_blobs.doc/generation.md")]
    generation: Cell<u64>,
}

impl ImageBlobs {
    #[doc = include_str!("image_blobs.doc/begin_frame.md")]
    pub(super) fn begin_frame(&self) {
        let now = self.generation.get().wrapping_add(1);
        self.generation.set(now);
        let mut entries = self.entries.borrow_mut();
        if entries.is_empty() {
            return;
        }
        // An image off screen for two frames stops holding its pixels alive. The
        // same window vello's own atlas uses before it evicts, so a blob is never
        // dropped while the texture it named is still resident.
        entries.retain(|_, (_, seen)| now.saturating_sub(*seen) <= KEEP_GENERATIONS);
    }

    #[doc = include_str!("image_blobs.doc/data_for.md")]
    pub(super) fn data_for(&self, image: &SceneImage, alpha_type: ImageAlphaType) -> ImageData {
        let pixels = image.pixels();
        // The allocation itself is the identity. It cannot be recycled under us
        // while this cache holds it, because the entry owns a `Blob` that owns
        // the same `Arc` — so an address that matches is the same pixels.
        let key = (pixels.as_ptr() as usize, pixels.len());
        let now = self.generation.get();
        let mut entries = self.entries.borrow_mut();
        if let Some((data, seen)) = entries.get_mut(&key)
            && data.alpha_type == alpha_type
        {
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
