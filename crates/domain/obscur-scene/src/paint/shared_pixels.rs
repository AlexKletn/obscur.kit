#![doc = include_str!("../../docs/api/paint/shared_pixels.doc/_module.md")]

use std::sync::Arc;

/// Bytes shared as a slice — what a decoder or any other `Arc<[u8]>` producer
/// already holds.
#[derive(Debug, Clone)]
pub enum SharedPixels {
    /// An `Arc<[u8]>` that already existed. Taken as-is.
    Shared(Arc<[u8]>),
    /// A `Vec` moved in. Taken as-is: converting it to an `Arc<[u8]>` would allocate a
    /// second buffer and copy the whole thing.
    Owned(Arc<Vec<u8>>),
}

impl SharedPixels {
    /// The bytes, whichever shape they are in.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Shared(bytes) => bytes,
            Self::Owned(bytes) => bytes,
        }
    }

    /// How many bytes there are.
    #[must_use]
    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    /// Whether there are no bytes at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }
}

impl AsRef<[u8]> for SharedPixels {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Arc<[u8]>> for SharedPixels {
    fn from(bytes: Arc<[u8]>) -> Self {
        Self::Shared(bytes)
    }
}

impl From<Vec<u8>> for SharedPixels {
    /// Moves the buffer in — it is never copied.
    fn from(bytes: Vec<u8>) -> Self {
        // A MOVE. `Arc<[u8]>::from(vec)` would allocate a second buffer and copy
        // into it — measured at 1080p, six milliseconds a frame to publish bytes
        // that were already sitting there.
        Self::Owned(Arc::new(bytes))
    }
}

impl PartialEq for SharedPixels {
    fn eq(&self, other: &Self) -> bool {
        // The same bytes are the same pixels, whichever shape they arrived in.
        // Pointer identity first, so comparing an image with itself — which is
        // what an equality check on a retained scene mostly does — never walks
        // eight megabytes.
        std::ptr::eq(self.as_bytes(), other.as_bytes()) || self.as_bytes() == other.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::SharedPixels;
    use std::sync::Arc;

    #[test]
    fn both_shapes_read_as_the_same_bytes() {
        let shared = SharedPixels::from(Arc::<[u8]>::from(vec![1, 2, 3]));
        let owned = SharedPixels::from(vec![1, 2, 3]);
        assert_eq!(shared.as_bytes(), owned.as_bytes());
        assert_eq!(shared, owned, "the same pixels, whichever shape");
        assert_eq!(owned.len(), 3);
        assert!(!owned.is_empty());
    }

    #[test]
    fn a_vec_is_moved_in_rather_than_copied() {
        // The whole reason this type exists. `Arc<[u8]>::from(vec)` allocates a
        // second buffer and copies; `Arc::new(vec)` takes the one that is
        // already there.
        let bytes = vec![7u8; 64];
        let address = bytes.as_ptr();
        let pixels = SharedPixels::from(bytes);
        assert!(
            std::ptr::eq(pixels.as_bytes().as_ptr(), address),
            "the buffer never moved"
        );
    }

    #[test]
    fn cloning_shares_rather_than_duplicates() {
        for pixels in [
            SharedPixels::from(vec![9u8; 32]),
            SharedPixels::from(Arc::<[u8]>::from(vec![9u8; 32])),
        ] {
            let clone = pixels.clone();
            assert!(
                std::ptr::eq(pixels.as_bytes().as_ptr(), clone.as_bytes().as_ptr()),
                "a clone is a refcount, not a copy"
            );
        }
    }

    #[test]
    fn different_bytes_are_not_equal() {
        assert_ne!(
            SharedPixels::from(vec![1, 2]),
            SharedPixels::from(vec![1, 3])
        );
    }
}
