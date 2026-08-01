#![doc = include_str!("document_handle.doc/_module.md")]

use crate::document::{DocumentMetadata, DocumentPermission};

#[doc = include_str!("document_handle.doc/DocumentHandle.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentHandle {
    token: Box<str>,
    metadata: DocumentMetadata,
    permission: DocumentPermission,
    path_hint: Option<Box<str>>,
}

impl DocumentHandle {
    #[doc = include_str!("document_handle.doc/new.md")]
    #[must_use]
    pub fn new(
        token: impl Into<Box<str>>,
        metadata: DocumentMetadata,
        permission: DocumentPermission,
    ) -> Self {
        Self {
            token: token.into(),
            metadata,
            permission,
            path_hint: None,
        }
    }

    #[doc = include_str!("document_handle.doc/with_path_hint.md")]
    #[must_use]
    pub fn with_path_hint(mut self, path_hint: impl Into<Box<str>>) -> Self {
        self.path_hint = Some(path_hint.into());
        self
    }

    #[doc = include_str!("document_handle.doc/token.md")]
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    #[doc = include_str!("document_handle.doc/metadata.md")]
    #[must_use]
    pub const fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }

    #[doc = include_str!("document_handle.doc/permission.md")]
    #[must_use]
    pub const fn permission(&self) -> &DocumentPermission {
        &self.permission
    }

    #[doc = include_str!("document_handle.doc/path_hint.md")]
    #[must_use]
    pub fn path_hint(&self) -> Option<&str> {
        self.path_hint.as_deref()
    }

    #[doc = include_str!("document_handle.doc/can_write.md")]
    #[must_use]
    pub const fn can_write(&self) -> bool {
        self.permission.can_write()
    }

    #[doc = include_str!("document_handle.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        self.metadata.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handle() -> DocumentHandle {
        DocumentHandle::new(
            "tok-1",
            DocumentMetadata::new("Notes.txt"),
            DocumentPermission::read_write(),
        )
    }

    #[test]
    fn accessors_expose_construction() {
        let h = handle();
        assert_eq!(h.token(), "tok-1");
        assert_eq!(h.metadata().name(), "Notes.txt");
        assert_eq!(h.permission(), &DocumentPermission::read_write());
        assert_eq!(h.path_hint(), None);
    }

    #[test]
    fn with_path_hint_sets_hint() {
        let h = handle().with_path_hint("/home/user/Notes.txt");
        assert_eq!(h.path_hint(), Some("/home/user/Notes.txt"));
    }

    #[test]
    fn delegates_can_write_and_name() {
        let writable = handle();
        assert!(writable.can_write());
        assert_eq!(writable.name(), "Notes.txt");

        let readonly = DocumentHandle::new(
            "tok-2",
            DocumentMetadata::new("ReadMe"),
            DocumentPermission::read_only(),
        );
        assert!(!readonly.can_write());
    }
}
