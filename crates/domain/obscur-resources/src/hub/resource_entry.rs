//! The Resource Hub's per-resource bookkeeping record (spec §5.15).

use crate::handle::{ResourceHandle, ResourceKind};
use crate::hub::ResourceNamespace;
use crate::source::ResourceSource;
use crate::state::{ResourceState, ResourceVersion};

/// One resource's complete record inside the [`crate::hub::ResourceHub`]
/// (spec §5.15).
///
/// An entry binds a resource's identity ([`ResourceHandle`]), its origin
/// ([`ResourceSource`]) and category ([`ResourceKind`]), its owning
/// [`ResourceNamespace`] scope, its current load lifecycle [`ResourceState`], the
/// authoritative content [`ResourceVersion`], and a `ref_count` recording how many
/// times the deduplicated resource has been requested.
///
/// The `version` field is the linchpin of **stale-job rejection**: it is the
/// version the hub currently considers authoritative. A load job captures this
/// value when it begins and presents it back on completion; if a hot reload has
/// since advanced `version`, the returning job is stale and is rejected (see
/// [`crate::hub::ResourceHub::complete`]).
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceEntry {
    /// The identity handle for this resource.
    handle: ResourceHandle,
    /// Where the resource's bytes come from.
    source: ResourceSource,
    /// The category of resource.
    kind: ResourceKind,
    /// The disposal / hot-reload scope that owns this resource.
    namespace: ResourceNamespace,
    /// The current load lifecycle state.
    state: ResourceState,
    /// The authoritative content version; a load job is rejected on completion
    /// if it no longer matches (spec §5.15 stale-job rejection).
    version: ResourceVersion,
    /// How many outstanding requests reference this deduplicated resource.
    ref_count: usize,
}

impl ResourceEntry {
    /// Creates a fresh entry for a newly requested resource (spec §5.15).
    ///
    /// The entry starts [`ResourceState::pending`] at [`ResourceVersion::initial`]
    /// with a reference count of `1`.
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

    /// Records one more request referencing this deduplicated resource (spec
    /// §5.15 deduplication).
    pub fn retain(&mut self) {
        self.ref_count = self.ref_count.saturating_add(1);
    }

    /// Drops one reference and returns the new count (spec §5.15).
    ///
    /// The count saturates at `0`; a return of `0` signals the resource is now
    /// unreferenced and may be evicted by the hub.
    pub fn release(&mut self) -> usize {
        self.ref_count = self.ref_count.saturating_sub(1);
        self.ref_count
    }

    /// Returns whether no requests reference this resource any longer.
    #[must_use]
    pub const fn is_unreferenced(&self) -> bool {
        self.ref_count == 0
    }

    /// Returns the identity handle for this resource.
    #[must_use]
    pub const fn handle(&self) -> ResourceHandle {
        self.handle
    }

    /// Returns where the resource's bytes come from.
    #[must_use]
    pub const fn source(&self) -> &ResourceSource {
        &self.source
    }

    /// Returns the category of resource.
    #[must_use]
    pub const fn kind(&self) -> ResourceKind {
        self.kind
    }

    /// Returns the disposal / hot-reload scope that owns this resource.
    #[must_use]
    pub const fn namespace(&self) -> &ResourceNamespace {
        &self.namespace
    }

    /// Returns the current load lifecycle state.
    #[must_use]
    pub const fn state(&self) -> &ResourceState {
        &self.state
    }

    /// Returns the authoritative content version (spec §5.15 versioning).
    #[must_use]
    pub const fn version(&self) -> ResourceVersion {
        self.version
    }

    /// Returns how many outstanding requests reference this resource.
    #[must_use]
    pub const fn ref_count(&self) -> usize {
        self.ref_count
    }

    /// Replaces the load lifecycle state (used by the hub's state machine).
    pub(crate) fn set_state(&mut self, state: ResourceState) {
        self.state = state;
    }

    /// Advances the authoritative version by one and returns it (spec §5.15 hot
    /// reload). Any in-flight job stamped with the prior version becomes stale.
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
