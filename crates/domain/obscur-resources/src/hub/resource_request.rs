//! An intent to acquire a resource from the Resource Hub (spec §5.15).

use crate::handle::ResourceKind;
use crate::hub::ResourceNamespace;
use crate::source::ResourceSource;

/// A declarative intent to acquire a resource (spec §5.15).
///
/// A request names *where* a resource comes from ([`ResourceSource`]), *what*
/// category it is ([`ResourceKind`]), and the [`ResourceNamespace`] scope that
/// owns it for disposal and hot reload. Submitting a request to the hub yields a
/// [`crate::handle::ResourceHandle`]; the request itself performs no I/O.
///
/// # Deduplication
///
/// Two requests denote the **same** resource when they share the same source
/// *and* the same kind; the namespace is ownership metadata and does not affect
/// identity. The hub deduplicates on [`ResourceRequest::dedup_key`], so a second
/// request for an already-tracked `(source, kind)` returns the existing handle
/// and bumps its reference count rather than allocating a new resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRequest {
    /// Where the resource's bytes come from.
    source: ResourceSource,
    /// The category of resource being requested.
    kind: ResourceKind,
    /// The disposal / hot-reload scope that owns the resource.
    namespace: ResourceNamespace,
}

impl ResourceRequest {
    /// Creates a request for `source` of the given `kind`, owned by the implicit
    /// global namespace (spec §5.15).
    ///
    /// Use [`ResourceRequest::in_namespace`] to scope it to a specific
    /// [`ResourceNamespace`].
    #[must_use]
    pub fn new(source: ResourceSource, kind: ResourceKind) -> Self {
        Self {
            source,
            kind,
            namespace: ResourceNamespace::DEFAULT,
        }
    }

    /// Returns this request scoped to `namespace` for disposal and hot reload
    /// (spec §5.15).
    #[must_use]
    pub fn in_namespace(mut self, namespace: ResourceNamespace) -> Self {
        self.namespace = namespace;
        self
    }

    /// Returns where the resource's bytes come from.
    #[must_use]
    pub const fn source(&self) -> &ResourceSource {
        &self.source
    }

    /// Returns the category of resource requested.
    #[must_use]
    pub const fn kind(&self) -> ResourceKind {
        self.kind
    }

    /// Returns the disposal / hot-reload scope that owns the resource.
    #[must_use]
    pub const fn namespace(&self) -> &ResourceNamespace {
        &self.namespace
    }

    /// Returns the identity key the hub deduplicates on: `(source, kind)`
    /// (spec §5.15 deduplication).
    ///
    /// The namespace is deliberately excluded — it is ownership metadata, not
    /// identity — so requests for the same bytes-and-category collapse to one
    /// resource regardless of scope.
    #[must_use]
    pub const fn dedup_key(&self) -> (&ResourceSource, ResourceKind) {
        (&self.source, self.kind)
    }

    /// Consumes the request into its owned parts, letting the hub build an entry
    /// without cloning.
    pub(crate) fn into_parts(self) -> (ResourceSource, ResourceKind, ResourceNamespace) {
        (self.source, self.kind, self.namespace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::MemorySource;
    use obscur_identity::NamespaceId;

    fn memory(bytes: &[u8]) -> ResourceSource {
        ResourceSource::memory(MemorySource::new(bytes.to_vec()))
    }

    #[test]
    fn new_defaults_to_global_namespace() {
        let req = ResourceRequest::new(memory(&[1, 2, 3]), ResourceKind::Image);
        assert_eq!(req.source(), &memory(&[1, 2, 3]));
        assert_eq!(req.kind(), ResourceKind::Image);
        assert!(req.namespace().is_global());
    }

    #[test]
    fn in_namespace_overrides_scope() {
        let ns = ResourceNamespace::new(NamespaceId::from_raw(5));
        let req = ResourceRequest::new(memory(&[9]), ResourceKind::Font).in_namespace(ns.clone());
        assert_eq!(req.namespace(), &ns);
    }

    #[test]
    fn dedup_key_is_source_and_kind() {
        let req = ResourceRequest::new(memory(&[1]), ResourceKind::Shader);
        assert_eq!(req.dedup_key(), (&memory(&[1]), ResourceKind::Shader));
    }

    #[test]
    fn dedup_key_ignores_namespace() {
        let global = ResourceRequest::new(memory(&[7]), ResourceKind::Image);
        let scoped = ResourceRequest::new(memory(&[7]), ResourceKind::Image)
            .in_namespace(ResourceNamespace::new(NamespaceId::from_raw(2)));
        assert_eq!(global.dedup_key(), scoped.dedup_key());
    }

    #[test]
    fn differing_source_or_kind_are_distinct_keys() {
        let a = ResourceRequest::new(memory(&[1]), ResourceKind::Image);
        let b = ResourceRequest::new(memory(&[2]), ResourceKind::Image);
        let c = ResourceRequest::new(memory(&[1]), ResourceKind::Font);
        assert_ne!(a.dedup_key(), b.dedup_key());
        assert_ne!(a.dedup_key(), c.dedup_key());
    }
}
