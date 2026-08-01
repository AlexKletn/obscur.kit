#![doc = include_str!("resource_entry.doc/_module.md")]

use crate::handle::{ResourceHandle, ResourceKind};
use crate::hub::ResourceNamespace;
use crate::source::ResourceSource;
use crate::state::{ResourceState, ResourceVersion};

#[doc = include_str!("resource_entry.doc/ResourceEntry.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceEntry {
    #[doc = include_str!("resource_entry.doc/handle.md")]
    handle: ResourceHandle,
    #[doc = include_str!("resource_entry.doc/source.md")]
    source: ResourceSource,
    #[doc = include_str!("resource_entry.doc/kind.md")]
    kind: ResourceKind,
    #[doc = include_str!("resource_entry.doc/namespace.md")]
    namespace: ResourceNamespace,
    #[doc = include_str!("resource_entry.doc/state.md")]
    state: ResourceState,
    #[doc = include_str!("resource_entry.doc/version.md")]
    version: ResourceVersion,
    #[doc = include_str!("resource_entry.doc/ref_count.md")]
    ref_count: usize,
}

impl ResourceEntry {
    #[doc = include_str!("resource_entry.doc/new.md")]
    #[must_use]
    pub fn new(
        handle: ResourceHandle,
        source: ResourceSource,
        kind: ResourceKind,
        namespace: ResourceNamespace,
    ) -> Self {
        Self {
            handle,
            source,
            kind,
            namespace,
            state: ResourceState::pending(),
            version: ResourceVersion::initial(),
            ref_count: 1,
        }
    }

    #[doc = include_str!("resource_entry.doc/retain.md")]
    pub fn retain(&mut self) {
        self.ref_count = self.ref_count.saturating_add(1);
    }

    #[doc = include_str!("resource_entry.doc/release.md")]
    pub fn release(&mut self) -> usize {
        self.ref_count = self.ref_count.saturating_sub(1);
        self.ref_count
    }

    #[doc = include_str!("resource_entry.doc/is_unreferenced.md")]
    #[must_use]
    pub const fn is_unreferenced(&self) -> bool {
        self.ref_count == 0
    }

    #[doc = include_str!("resource_entry.doc/handle_2.md")]
    #[must_use]
    pub const fn handle(&self) -> ResourceHandle {
        self.handle
    }

    #[doc = include_str!("resource_entry.doc/source_2.md")]
    #[must_use]
    pub const fn source(&self) -> &ResourceSource {
        &self.source
    }

    #[doc = include_str!("resource_entry.doc/kind_2.md")]
    #[must_use]
    pub const fn kind(&self) -> ResourceKind {
        self.kind
    }

    #[doc = include_str!("resource_entry.doc/namespace_2.md")]
    #[must_use]
    pub const fn namespace(&self) -> &ResourceNamespace {
        &self.namespace
    }

    #[doc = include_str!("resource_entry.doc/state_2.md")]
    #[must_use]
    pub const fn state(&self) -> &ResourceState {
        &self.state
    }

    #[doc = include_str!("resource_entry.doc/version_2.md")]
    #[must_use]
    pub const fn version(&self) -> ResourceVersion {
        self.version
    }

    #[doc = include_str!("resource_entry.doc/ref_count_2.md")]
    #[must_use]
    pub const fn ref_count(&self) -> usize {
        self.ref_count
    }

    #[doc = include_str!("resource_entry.doc/set_state.md")]
    pub(crate) fn set_state(&mut self, state: ResourceState) {
        self.state = state;
    }

    #[doc = include_str!("resource_entry.doc/bump_version.md")]
    pub(crate) fn bump_version(&mut self) -> ResourceVersion {
        self.version = self.version.next();
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::MemorySource;
    use crate::state::{ResourceError, ResourceMetadata};
    use obscur_identity::ResourceId;

    fn entry() -> ResourceEntry {
        let handle = ResourceHandle::new(ResourceId::from_raw(1), ResourceKind::Image);
        let source = ResourceSource::memory(MemorySource::new(vec![1u8, 2, 3]));
        ResourceEntry::new(
            handle,
            source,
            ResourceKind::Image,
            ResourceNamespace::DEFAULT,
        )
    }

    #[test]
    fn new_starts_pending_initial_ref_one() {
        let e = entry();
        assert_eq!(e.state(), &ResourceState::pending());
        assert_eq!(e.version(), ResourceVersion::initial());
        assert_eq!(e.ref_count(), 1);
        assert!(!e.is_unreferenced());
        assert_eq!(e.kind(), ResourceKind::Image);
        assert!(e.namespace().is_global());
    }

    #[test]
    fn retain_and_release_track_ref_count() {
        let mut e = entry();
        e.retain();
        assert_eq!(e.ref_count(), 2);
        assert_eq!(e.release(), 1);
        assert!(!e.is_unreferenced());
        assert_eq!(e.release(), 0);
        assert!(e.is_unreferenced());
    }

    #[test]
    fn release_saturates_at_zero() {
        let mut e = entry();
        assert_eq!(e.release(), 0);
        assert_eq!(e.release(), 0);
        assert!(e.is_unreferenced());
    }

    #[test]
    fn set_state_and_bump_version() {
        let mut e = entry();
        let v2 = e.bump_version();
        assert_eq!(v2, ResourceVersion::initial().next());
        assert_eq!(e.version(), v2);

        e.set_state(ResourceState::failed(ResourceError::NotFound, v2));
        assert!(e.state().is_failed());
        assert_eq!(e.state().version(), Some(v2));

        e.set_state(ResourceState::ready(v2, ResourceMetadata::new()));
        assert!(e.state().is_ready());
    }
}
