#![doc = include_str!("font_handle.doc/_module.md")]

use obscur_identity::ResourceId;

use crate::handle::{ResourceHandle, ResourceKind};

#[doc = include_str!("font_handle.doc/FontHandle.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct FontHandle(ResourceId);

impl FontHandle {
    #[doc = include_str!("font_handle.doc/new.md")]
    pub const fn new(id: ResourceId) -> Self {
        Self(id)
    }

    #[doc = include_str!("font_handle.doc/id.md")]
    #[must_use]
    pub const fn id(self) -> ResourceId {
        self.0
    }

    #[doc = include_str!("font_handle.doc/kind.md")]
    #[must_use]
    pub const fn kind() -> ResourceKind {
        ResourceKind::Font
    }

    #[doc = include_str!("font_handle.doc/into_handle.md")]
    #[must_use]
    pub const fn into_handle(self) -> ResourceHandle {
        ResourceHandle::new(self.0, ResourceKind::Font)
    }

    #[doc = include_str!("font_handle.doc/from_handle.md")]
    #[must_use]
    pub const fn from_handle(handle: ResourceHandle) -> Option<Self> {
        match handle.kind() {
            ResourceKind::Font => Some(Self(handle.id())),
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
    fn kind_is_font() {
        assert_eq!(FontHandle::kind(), ResourceKind::Font);
    }

    #[test]
    fn round_trips_through_generic_handle() {
        let h = FontHandle::new(id(3));
        assert_eq!(h.id(), id(3));
        assert_eq!(h.into_handle().as_font(), Some(h));
        assert_eq!(FontHandle::from_handle(h.into_handle()), Some(h));
    }

    #[test]
    fn from_handle_rejects_mismatched_kind() {
        let image = ResourceHandle::new(id(3), ResourceKind::Image);
        assert_eq!(FontHandle::from_handle(image), None);
    }

    #[test]
    fn equality_and_hash_track_id() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = FontHandle::new(id(1));
        let b = FontHandle::new(id(1));
        let c = FontHandle::new(id(2));
        assert_eq!(a, b);
        assert_ne!(a, c);

        let hash = |h: FontHandle| {
            let mut s = DefaultHasher::new();
            h.hash(&mut s);
            s.finish()
        };
        assert_eq!(hash(a), hash(b));
    }
}
