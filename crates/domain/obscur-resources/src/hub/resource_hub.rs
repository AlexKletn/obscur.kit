#![doc = include_str!("resource_hub.doc/_module.md")]

use std::collections::HashMap;

use obscur_identity::ResourceId;

use crate::handle::{ResourceHandle, ResourceKind};
use crate::hub::{ResourceEntry, ResourceNamespace, ResourceRequest};
use crate::source::ResourceSource;
use crate::state::{ResourceError, ResourceMetadata, ResourceState, ResourceVersion};

#[doc = include_str!("resource_hub.doc/ResourceHub.md")]
#[derive(Debug, Clone, Default)]
pub struct ResourceHub {
    #[doc = include_str!("resource_hub.doc/entries.md")]
    entries: HashMap<ResourceId, ResourceEntry>,
    #[doc = include_str!("resource_hub.doc/by_source.md")]
    by_source: HashMap<(ResourceSource, ResourceKind), ResourceId>,
    #[doc = include_str!("resource_hub.doc/next_id.md")]
    next_id: u64,
}

impl ResourceHub {
    #[doc = include_str!("resource_hub.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("resource_hub.doc/allocate_id.md")]
    fn allocate_id(&mut self) -> ResourceId {
        let id = ResourceId::from_raw(self.next_id);
        self.next_id += 1;
        id
    }

    #[doc = include_str!("resource_hub.doc/request.md")]
    pub fn request(&mut self, request: ResourceRequest) -> ResourceHandle {
        if let Some(&id) = self
            .by_source
            .get(&(request.source().clone(), request.kind()))
            && let Some(entry) = self.entries.get_mut(&id)
        {
            entry.retain();
            return entry.handle();
        }

        let (source, kind, namespace) = request.into_parts();
        let id = self.allocate_id();
        let handle = ResourceHandle::new(id, kind);
        let entry = ResourceEntry::new(handle, source.clone(), kind, namespace);
        self.entries.insert(id, entry);
        self.by_source.insert((source, kind), id);
        handle
    }

    #[doc = include_str!("resource_hub.doc/handle_for.md")]
    #[must_use]
    pub fn handle_for(
        &self,
        source: &ResourceSource,
        kind: ResourceKind,
    ) -> Option<ResourceHandle> {
        let id = self.by_source.get(&(source.clone(), kind))?;
        self.entries.get(id).map(ResourceEntry::handle)
    }

    #[doc = include_str!("resource_hub.doc/entry.md")]
    #[must_use]
    pub fn entry(&self, handle: ResourceHandle) -> Option<&ResourceEntry> {
        self.entries.get(&handle.id())
    }

    #[doc = include_str!("resource_hub.doc/state.md")]
    #[must_use]
    pub fn state(&self, handle: ResourceHandle) -> Option<&ResourceState> {
        self.entries.get(&handle.id()).map(ResourceEntry::state)
    }

    #[doc = include_str!("resource_hub.doc/begin_load.md")]
    pub fn begin_load(&mut self, handle: ResourceHandle) -> Option<ResourceVersion> {
        let entry = self.entries.get_mut(&handle.id())?;
        entry.set_state(ResourceState::loading(None));
        Some(entry.version())
    }

    #[doc = include_str!("resource_hub.doc/set_progress.md")]
    pub fn set_progress(&mut self, handle: ResourceHandle, progress: Option<f32>) {
        if let Some(entry) = self.entries.get_mut(&handle.id()) {
            entry.set_state(ResourceState::loading(progress));
        }
    }

    #[doc = include_str!("resource_hub.doc/complete.md")]
    pub fn complete(
        &mut self,
        handle: ResourceHandle,
        job_version: ResourceVersion,
        metadata: ResourceMetadata,
    ) -> bool {
        let Some(entry) = self.entries.get_mut(&handle.id()) else {
            return false;
        };
        if job_version != entry.version() {
            return false;
        }
        entry.set_state(ResourceState::ready(entry.version(), metadata));
        true
    }

    #[doc = include_str!("resource_hub.doc/fail.md")]
    pub fn fail(
        &mut self,
        handle: ResourceHandle,
        job_version: ResourceVersion,
        error: ResourceError,
    ) -> bool {
        let Some(entry) = self.entries.get_mut(&handle.id()) else {
            return false;
        };
        if job_version != entry.version() {
            return false;
        }
        entry.set_state(ResourceState::failed(error, entry.version()));
        true
    }

    #[doc = include_str!("resource_hub.doc/reload.md")]
    pub fn reload(&mut self, handle: ResourceHandle) -> Option<ResourceVersion> {
        let entry = self.entries.get_mut(&handle.id())?;
        let version = entry.bump_version();
        entry.set_state(ResourceState::loading(None));
        Some(version)
    }

    #[doc = include_str!("resource_hub.doc/release.md")]
    pub fn release(&mut self, handle: ResourceHandle) -> bool {
        let Some(entry) = self.entries.get_mut(&handle.id()) else {
            return false;
        };
        if entry.release() == 0 {
            self.evict(handle.id());
            true
        } else {
            false
        }
    }

    #[doc = include_str!("resource_hub.doc/evict.md")]
    fn evict(&mut self, id: ResourceId) {
        if let Some(entry) = self.entries.remove(&id) {
            self.by_source
                .remove(&(entry.source().clone(), entry.kind()));
        }
    }

    #[doc = include_str!("resource_hub.doc/dispose_namespace.md")]
    pub fn dispose_namespace(&mut self, namespace: &ResourceNamespace) -> usize {
        let doomed: Vec<ResourceId> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.namespace() == namespace)
            .map(|(id, _)| *id)
            .collect();
        for id in &doomed {
            self.evict(*id);
        }
        doomed.len()
    }

    #[doc = include_str!("resource_hub.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[doc = include_str!("resource_hub.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[doc = include_str!("resource_hub.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, handle: ResourceHandle) -> bool {
        self.entries.contains_key(&handle.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::MemorySource;
    use obscur_identity::NamespaceId;

    fn source(bytes: &[u8]) -> ResourceSource {
        ResourceSource::memory(MemorySource::new(bytes.to_vec()))
    }

    fn request(bytes: &[u8], kind: ResourceKind) -> ResourceRequest {
        ResourceRequest::new(source(bytes), kind)
    }

    #[test]
    fn request_deduplicates_equal_source_and_kind() {
        let mut hub = ResourceHub::new();
        let a = hub.request(request(&[1, 2, 3], ResourceKind::Image));
        let b = hub.request(request(&[1, 2, 3], ResourceKind::Image));

        // Same identity, single entry, reference count of two.
        assert_eq!(a, b);
        assert_eq!(hub.len(), 1);
        assert_eq!(hub.entry(a).unwrap().ref_count(), 2);
        assert_eq!(
            hub.handle_for(&source(&[1, 2, 3]), ResourceKind::Image),
            Some(a)
        );
    }

    #[test]
    fn distinct_sources_and_kinds_allocate_distinct_ids() {
        let mut hub = ResourceHub::new();
        let a = hub.request(request(&[1], ResourceKind::Image));
        let b = hub.request(request(&[2], ResourceKind::Image));
        let c = hub.request(request(&[1], ResourceKind::Font));

        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
        assert_eq!(hub.len(), 3);
    }

    #[test]
    fn begin_load_then_complete_reaches_ready() {
        let mut hub = ResourceHub::new();
        let h = hub.request(request(&[1], ResourceKind::Image));
        assert_eq!(hub.state(h), Some(&ResourceState::pending()));

        let v = hub.begin_load(h).expect("known handle");
        assert_eq!(v, ResourceVersion::initial());
        assert!(hub.state(h).unwrap().is_loading());

        hub.set_progress(h, Some(0.5));
        assert_eq!(hub.state(h), Some(&ResourceState::loading(Some(0.5))));

        let meta = ResourceMetadata::new().with_byte_size(64);
        assert!(hub.complete(h, v, meta.clone()));
        assert_eq!(hub.state(h), Some(&ResourceState::ready(v, meta)));
    }

    #[test]
    fn stale_job_is_rejected_after_reload() {
        let mut hub = ResourceHub::new();
        let h = hub.request(request(&[1], ResourceKind::Shader));

        // A load job captures the initial version.
        let v1 = hub.begin_load(h).expect("known handle");

        // A hot reload supersedes it with a newer version.
        let v2 = hub.reload(h).expect("known handle");
        assert!(v2.is_newer_than(v1));
        assert!(hub.state(h).unwrap().is_loading());

        // The stale job for v1 is rejected and leaves the state loading.
        assert!(!hub.complete(h, v1, ResourceMetadata::new()));
        assert!(hub.state(h).unwrap().is_loading());

        // The current job for v2 succeeds.
        let meta = ResourceMetadata::new();
        assert!(hub.complete(h, v2, meta.clone()));
        assert_eq!(hub.state(h), Some(&ResourceState::ready(v2, meta)));
    }

    #[test]
    fn fail_applies_stale_check() {
        let mut hub = ResourceHub::new();
        let h = hub.request(request(&[1], ResourceKind::Image));
        let v1 = hub.begin_load(h).unwrap();
        let v2 = hub.reload(h).unwrap();

        // Stale failure is rejected.
        assert!(!hub.fail(h, v1, ResourceError::NotFound));
        assert!(hub.state(h).unwrap().is_loading());

        // Current failure is recorded at the current version.
        assert!(hub.fail(h, v2, ResourceError::NotFound));
        assert_eq!(
            hub.state(h),
            Some(&ResourceState::failed(ResourceError::NotFound, v2))
        );
    }

    #[test]
    fn reload_version_is_monotonic() {
        let mut hub = ResourceHub::new();
        let h = hub.request(request(&[1], ResourceKind::Image));
        let v1 = hub.entry(h).unwrap().version();
        let v2 = hub.reload(h).unwrap();
        let v3 = hub.reload(h).unwrap();
        assert!(v2.is_newer_than(v1));
        assert!(v3.is_newer_than(v2));
    }

    #[test]
    fn release_decrements_then_removes_at_zero() {
        let mut hub = ResourceHub::new();
        let h = hub.request(request(&[1], ResourceKind::Image));
        hub.request(request(&[1], ResourceKind::Image)); // ref_count -> 2

        // First release only decrements; still tracked.
        assert!(!hub.release(h));
        assert!(hub.contains(h));
        assert_eq!(hub.entry(h).unwrap().ref_count(), 1);

        // Second release evicts the entry and its dedup index.
        assert!(hub.release(h));
        assert!(!hub.contains(h));
        assert!(hub.is_empty());
        assert_eq!(hub.handle_for(&source(&[1]), ResourceKind::Image), None);
    }

    #[test]
    fn released_source_can_be_requested_fresh() {
        let mut hub = ResourceHub::new();
        let a = hub.request(request(&[1], ResourceKind::Image));
        assert!(hub.release(a));
        // A fresh request re-allocates a new identity for the same source.
        let b = hub.request(request(&[1], ResourceKind::Image));
        assert_ne!(a, b);
        assert_eq!(hub.entry(b).unwrap().ref_count(), 1);
    }

    #[test]
    fn release_unknown_handle_is_false() {
        let mut hub = ResourceHub::new();
        let ghost = ResourceHandle::new(ResourceId::from_raw(999), ResourceKind::Image);
        assert!(!hub.release(ghost));
        assert_eq!(hub.begin_load(ghost), None);
        assert_eq!(hub.reload(ghost), None);
        assert!(!hub.complete(ghost, ResourceVersion::initial(), ResourceMetadata::new()));
        assert!(!hub.fail(ghost, ResourceVersion::initial(), ResourceError::NotFound));
    }

    #[test]
    fn dispose_namespace_removes_scoped_entries_ignoring_ref_count() {
        let mut hub = ResourceHub::new();
        let doc = ResourceNamespace::new(NamespaceId::from_raw(1));

        let a = hub.request(request(&[1], ResourceKind::Image).in_namespace(doc.clone()));
        // Give it two references to prove disposal ignores ref_count.
        hub.request(request(&[1], ResourceKind::Image).in_namespace(doc.clone()));
        let b = hub.request(request(&[2], ResourceKind::Font).in_namespace(doc.clone()));
        // A resource in the global namespace must survive.
        let g = hub.request(request(&[3], ResourceKind::Image));

        let removed = hub.dispose_namespace(&doc);
        assert_eq!(removed, 2);
        assert!(!hub.contains(a));
        assert!(!hub.contains(b));
        assert!(hub.contains(g));
        assert_eq!(hub.len(), 1);
        // The dedup indices for the disposed resources are gone too.
        assert_eq!(hub.handle_for(&source(&[1]), ResourceKind::Image), None);
        assert_eq!(hub.handle_for(&source(&[2]), ResourceKind::Font), None);
    }

    #[test]
    fn len_is_empty_and_contains_track_registry() {
        let mut hub = ResourceHub::new();
        assert!(hub.is_empty());
        let h = hub.request(request(&[1], ResourceKind::Image));
        assert!(!hub.is_empty());
        assert_eq!(hub.len(), 1);
        assert!(hub.contains(h));
    }
}
