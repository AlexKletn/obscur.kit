//! A backend-neutral capture of a rendered raster image: [`ImageSnapshot`].

/// A deterministic, backend-neutral capture of a rendered raster image
/// (spec §7.4).
///
/// The rendered image is the one §7.4 stage that is *backend-dependent*: pixels
/// come out of a concrete rasterizer, not the backend-independent pipeline. To
/// keep `obscur-headless` free of any GPU or rasterizer dependency, this type
/// does **not** hold or produce pixels itself. Instead it records the image's
/// dimensions and a stable **content hash** of its bytes, built with
/// [`from_rgba`](Self::from_rgba) from whatever raster a backend produced (for
/// example the software `obscur-render-cpu` backend's
/// `tiny_skia::Pixmap::data`).
///
/// # Determinism (spec §7.14)
///
/// The hash is a plain FNV-1a over the raw bytes, so identical dimensions and
/// bytes always produce an equal snapshot. A software backend rasterizing the
/// same [`SceneSnapshot`](obscur_scene::scene::SceneSnapshot) into the same-sized
/// target twice therefore yields equal image snapshots — the property a
/// golden-image test asserts without embedding the whole pixel buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSnapshot {
    /// The image width in physical pixels.
    width: u32,
    /// The image height in physical pixels.
    height: u32,
    /// A stable FNV-1a content hash of the image's raw bytes.
    pixel_hash: u64,
}

impl ImageSnapshot {
    /// The FNV-1a 64-bit offset basis.
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    /// The FNV-1a 64-bit prime.
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    /// Captures an image snapshot from raw pixel `bytes` of a `width` × `height`
    /// raster (any byte layout — RGBA8, BGRA8, premultiplied or not — as long as
    /// a given backend is consistent).
    ///
    /// The `bytes` are hashed with FNV-1a into a stable content digest;
    /// identical dimensions and bytes always produce an equal snapshot.
    #[must_use]
    pub fn from_rgba(width: u32, height: u32, bytes: &[u8]) -> Self {
        let mut hash = Self::FNV_OFFSET;
        for &byte in bytes {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(Self::FNV_PRIME);
        }
        Self {
            width,
            height,
            pixel_hash: hash,
        }
    }

    /// Builds a snapshot directly from an already-computed content hash, for
    /// callers that hashed the pixels themselves.
    #[must_use]
    pub const fn from_parts(width: u32, height: u32, pixel_hash: u64) -> Self {
        Self {
            width,
            height,
            pixel_hash,
        }
    }

    /// The image width in physical pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// The image height in physical pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The stable content hash of the image's bytes.
    #[must_use]
    pub const fn pixel_hash(&self) -> u64 {
        self.pixel_hash
    }

    /// Whether the image has zero area.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_bytes_hash_equal() {
        let a = ImageSnapshot::from_rgba(2, 2, &[1, 2, 3, 4, 5, 6, 7, 8]);
        let b = ImageSnapshot::from_rgba(2, 2, &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(a, b);
        assert_eq!(a.width(), 2);
        assert_eq!(a.height(), 2);
        assert!(!a.is_empty());
    }

    #[test]
    fn different_bytes_hash_differently() {
        let a = ImageSnapshot::from_rgba(2, 2, &[1, 2, 3, 4]);
        let b = ImageSnapshot::from_rgba(2, 2, &[1, 2, 3, 5]);
        assert_ne!(a.pixel_hash(), b.pixel_hash());
    }

    #[test]
    fn empty_dimensions_report_empty() {
        assert!(ImageSnapshot::from_rgba(0, 4, &[]).is_empty());
        assert!(ImageSnapshot::from_rgba(4, 0, &[]).is_empty());
        assert!(!ImageSnapshot::from_rgba(1, 1, &[0, 0, 0, 0]).is_empty());
    }

    #[test]
    fn from_parts_round_trips_fields() {
        let img = ImageSnapshot::from_parts(16, 9, 0xdead_beef);
        assert_eq!(img.width(), 16);
        assert_eq!(img.height(), 9);
        assert_eq!(img.pixel_hash(), 0xdead_beef);
    }
}
