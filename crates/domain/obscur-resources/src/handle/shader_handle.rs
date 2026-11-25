//! A type-safe handle denoting a shader resource (spec §5.15).

use obscur_identity::ResourceId;

use crate::handle::{ResourceHandle, ResourceKind};

#[doc = include_str!("../../docs/api/handle/shader_handle.doc/ShaderHandle.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct ShaderHandle(ResourceId);

impl ShaderHandle {
    /// Creates a shader handle for the given resource id.
    pub const fn new(id: ResourceId) -> Self {
        Self(id)
    }

    /// Returns the resource identity this handle refers to.
    #[must_use]
    pub const fn id(self) -> ResourceId {
        self.0
    }

    /// Returns the [`ResourceKind`] this handle type denotes ([`ResourceKind::Shader`]).
    #[must_use]
    pub const fn kind() -> ResourceKind {
        ResourceKind::Shader
    }

    /// Converts this typed handle into a generic [`ResourceHandle`].
    #[must_use]
    pub const fn into_handle(self) -> ResourceHandle {
        ResourceHandle::new(self.0, ResourceKind::Shader)
    }

    /// Recovers a `ShaderHandle` from a generic [`ResourceHandle`], or `None` if
    /// the handle does not denote a shader.
    #[must_use]
    pub const fn from_handle(handle: ResourceHandle) -> Option<Self> {
        match handle.kind() {
            ResourceKind::Shader => Some(Self(handle.id())),
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
    fn kind_is_shader() {
        assert_eq!(ShaderHandle::kind(), ResourceKind::Shader);
    }

    #[test]
    fn round_trips_through_generic_handle() {
        let h = ShaderHandle::new(id(3));
        assert_eq!(h.id(), id(3));
        assert_eq!(h.into_handle().as_shader(), Some(h));
        assert_eq!(ShaderHandle::from_handle(h.into_handle()), Some(h));
    }

    #[test]
    fn from_handle_rejects_mismatched_kind() {
        let font = ResourceHandle::new(id(3), ResourceKind::Font);
        assert_eq!(ShaderHandle::from_handle(font), None);
    }

    #[test]
    fn equality_and_hash_track_id() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = ShaderHandle::new(id(1));
        let b = ShaderHandle::new(id(1));
        let c = ShaderHandle::new(id(2));
        assert_eq!(a, b);
        assert_ne!(a, c);

        let hash = |h: ShaderHandle| {
            let mut s = DefaultHasher::new();
            h.hash(&mut s);
            s.finish()
        };
        assert_eq!(hash(a), hash(b));
    }
}
