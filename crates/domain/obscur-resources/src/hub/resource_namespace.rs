#![doc = include_str!("resource_namespace.doc/_module.md")]

use obscur_identity::NamespaceId;

#[doc = include_str!("resource_namespace.doc/ResourceNamespace.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceNamespace {
    #[doc = include_str!("resource_namespace.doc/id.md")]
    id: NamespaceId,
    #[doc = include_str!("resource_namespace.doc/label.md")]
    label: Option<Box<str>>,
}

impl ResourceNamespace {
    #[doc = include_str!("resource_namespace.doc/DEFAULT.md")]
    pub const DEFAULT: Self = Self {
        id: NamespaceId::from_raw(0),
        label: None,
    };

    #[doc = include_str!("resource_namespace.doc/new.md")]
    #[must_use]
    pub const fn new(id: NamespaceId) -> Self {
        Self { id, label: None }
    }

    #[doc = include_str!("resource_namespace.doc/with_label.md")]
    #[must_use]
    pub fn with_label(id: NamespaceId, label: impl Into<Box<str>>) -> Self {
        Self {
            id,
            label: Some(label.into()),
        }
    }

    #[doc = include_str!("resource_namespace.doc/default_namespace.md")]
    #[must_use]
    pub const fn default_namespace() -> Self {
        Self::DEFAULT
    }

    #[doc = include_str!("resource_namespace.doc/id_2.md")]
    #[must_use]
    pub const fn id(&self) -> NamespaceId {
        self.id
    }

    #[doc = include_str!("resource_namespace.doc/label_2.md")]
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    #[doc = include_str!("resource_namespace.doc/is_global.md")]
    #[must_use]
    pub const fn is_global(&self) -> bool {
        self.id.raw() == 0
    }
}

impl Default for ResourceNamespace {
    #[doc = include_str!("resource_namespace.doc/default.md")]
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
