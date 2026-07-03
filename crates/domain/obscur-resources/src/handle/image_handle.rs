//! A type-safe handle denoting an image resource (spec §5.15).

use obscur_identity::ResourceId;

use crate::handle::{ResourceHandle, ResourceKind};

/// A type-safe handle to an image resource (spec §5.15).
///
/// `ImageHandle` is a distinct newtype over [`ResourceId`] so that, at compile
/// time, it can only denote an image. It converts to and from the generic
/// [`ResourceHandle`] via [`Self::into_handle`] and [`Self::from_handle`]. Like
/// all handles it is a pure identifier: cheap, `Copy`, and owning no data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct ImageHandle(ResourceId);

impl ImageHandle {
    /// Creates an image handle for the given resource id.
    pub const fn new(id: ResourceId) -> Self {
        Self(id)
    }

    /// Returns the resource identity this handle refers to.
    #[must_use]
    pub const fn id(self) -> ResourceId {
        self.0
    }

    /// Returns the [`ResourceKind`] this handle type denotes ([`ResourceKind::Image`]).
    #[must_use]
    pub const fn kind() -> ResourceKind {
        ResourceKind::Image
    }

    /// Converts this typed handle into a generic [`ResourceHandle`].
    #[must_use]
    pub const fn into_handle(self) -> ResourceHandle {
        ResourceHandle::new(self.0, ResourceKind::Image)
    }

    /// Recovers an `ImageHandle` from a generic [`ResourceHandle`], or `None` if
    /// the handle does not denote an image.
    #[must_use]
    pub const fn from_handle(handle: ResourceHandle) -> Option<Self> {
        match handle.kind() {
            ResourceKind::Image => Some(Self(handle.id())),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: u64) -> ResourceId {
        ResourceId::from_raw(raw)
    }

    #[test]
    fn kind_is_image() {
        assert_eq!(ImageHandle::kind(), ResourceKind::Image);
    }

    #[test]
    fn round_trips_through_generic_handle() {
        let h = ImageHandle::new(id(3));
        assert_eq!(h.id(), id(3));
        assert_eq!(h.into_handle().as_image(), Some(h));
        assert_eq!(ImageHandle::from_handle(h.into_handle()), Some(h));
    }

    #[test]
    fn from_handle_rejects_mismatched_kind() {
        let font = ResourceHandle::new(id(3), ResourceKind::Font);
        assert_eq!(ImageHandle::from_handle(font), None);
    }

    #[test]
    fn equality_and_hash_track_id() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = ImageHandle::new(id(1));
        let b = ImageHandle::new(id(1));
        let c = ImageHandle::new(id(2));
        assert_eq!(a, b);
        assert_ne!(a, c);

        let hash = |h: ImageHandle| {
            let mut s = DefaultHasher::new();
            h.hash(&mut s);
            s.finish()
        };
        assert_eq!(hash(a), hash(b));
    }
}
