//! A disposal and hot-reload scope grouping resources (spec §5.15).

use obscur_identity::NamespaceId;

#[doc = include_str!("../../docs/api/hub/resource_namespace.doc/ResourceNamespace.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceNamespace {
    /// The opaque identity the hub keys disposal and hot reload on.
    id: NamespaceId,
    /// An optional human-readable label, for diagnostics only.
    label: Option<Box<str>>,
}

impl ResourceNamespace {
    /// The implicit global namespace (id raw `0`), owning every resource that is
    /// requested without an explicit scope (spec §5.15).
    pub const DEFAULT: Self = Self {
        id: NamespaceId::from_raw(0),
        label: None,
    };

    /// Creates an unlabeled namespace for the given identity.
    #[must_use]
    pub const fn new(id: NamespaceId) -> Self {
        Self { id, label: None }
    }

    /// Creates a namespace carrying a human-readable diagnostic label.
    #[must_use]
    pub fn with_label(id: NamespaceId, label: impl Into<Box<str>>) -> Self {
        Self {
            id,
            label: Some(label.into()),
        }
    }

    #[doc = include_str!("../../docs/api/hub/resource_namespace.doc/default_namespace.md")]
    #[must_use]
    pub const fn default_namespace() -> Self {
        Self::DEFAULT
    }

    /// Returns the opaque identity the hub keys on.
    #[must_use]
    pub const fn id(&self) -> NamespaceId {
        self.id
    }

    /// Returns the human-readable label, if any (diagnostics only).
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns whether this is the implicit global namespace (id raw `0`).
    #[must_use]
    pub const fn is_global(&self) -> bool {
        self.id.raw() == 0
    }
}

impl Default for ResourceNamespace {
    /// The default namespace is the implicit global one
    /// ([`ResourceNamespace::DEFAULT`]).
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_global_namespace_zero() {
        let ns = ResourceNamespace::default();
        assert_eq!(ns.id(), NamespaceId::from_raw(0));
        assert!(ns.is_global());
        assert_eq!(ns.label(), None);
        assert_eq!(ns, ResourceNamespace::DEFAULT);
        assert_eq!(ns, ResourceNamespace::default_namespace());
    }

    #[test]
    fn new_is_unlabeled() {
        let ns = ResourceNamespace::new(NamespaceId::from_raw(7));
        assert_eq!(ns.id(), NamespaceId::from_raw(7));
        assert_eq!(ns.label(), None);
        assert!(!ns.is_global());
    }

    #[test]
    fn with_label_carries_diagnostic_text() {
        let ns = ResourceNamespace::with_label(NamespaceId::from_raw(3), "document-42");
        assert_eq!(ns.id(), NamespaceId::from_raw(3));
        assert_eq!(ns.label(), Some("document-42"));
        assert!(!ns.is_global());
    }

    #[test]
    fn equality_and_hash_track_id_and_label() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let labeled = ResourceNamespace::with_label(NamespaceId::from_raw(1), "a");
        let same = ResourceNamespace::with_label(NamespaceId::from_raw(1), "a");
        let other_label = ResourceNamespace::with_label(NamespaceId::from_raw(1), "b");
        let unlabeled = ResourceNamespace::new(NamespaceId::from_raw(1));
        assert_eq!(labeled, same);
        assert_ne!(labeled, other_label);
        assert_ne!(labeled, unlabeled);

        let hash = |ns: &ResourceNamespace| {
            let mut hasher = DefaultHasher::new();
            ns.hash(&mut hasher);
            hasher.finish()
        };
        assert_eq!(hash(&labeled), hash(&same));
    }
}
