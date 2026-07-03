//! An opaque handle to a single document (spec §6.10 — not necessarily a path).

use crate::document::{DocumentMetadata, DocumentPermission};

/// An opaque handle to a single document a file picker returned
/// (spec §6.10 — a handle is **not** necessarily a filesystem path).
///
/// The handle works uniformly across local files, portals, cloud documents, and
/// virtual filesystems: its primary key is an opaque `token` the host resolves
/// back to the underlying document. The kit never reads the document itself here
/// (spec §5.15) — a [`DocumentService`](crate::document::DocumentService) adapter
/// resolves the token to bytes later.
///
/// Alongside the token, a handle carries descriptive [`DocumentMetadata`], the
/// granted [`DocumentPermission`], and an optional `path_hint`. The path hint is
/// **advisory only** (spec §6.10 optional path hint): a picker may expose only a
/// display name or a sandboxed identifier, so the hint is metadata for display or
/// logging and is *not* guaranteed to be a real filesystem path — never open it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentHandle {
    token: Box<str>,
    metadata: DocumentMetadata,
    permission: DocumentPermission,
    path_hint: Option<Box<str>>,
}

impl DocumentHandle {
    /// Creates a document handle from its opaque token, metadata, and permission
    /// (spec §6.10).
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

    /// Returns this handle with its advisory path hint set (spec §6.10).
    ///
    /// The hint is not guaranteed to be a real filesystem path; see the type docs.
    #[must_use]
    pub fn with_path_hint(mut self, path_hint: impl Into<Box<str>>) -> Self {
        self.path_hint = Some(path_hint.into());
        self
    }

    /// Returns the opaque host token identifying the document (spec §6.10).
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns the descriptive metadata (spec §6.10).
    #[must_use]
    pub const fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }

    /// Returns the granted permission (spec §6.10).
    #[must_use]
    pub const fn permission(&self) -> &DocumentPermission {
        &self.permission
    }

    /// Returns the advisory path hint, if any (spec §6.10 — not a real path).
    #[must_use]
    pub fn path_hint(&self) -> Option<&str> {
        self.path_hint.as_deref()
    }

    /// Returns whether writing is permitted, delegating to the permission
    /// (spec §6.10).
    #[must_use]
    pub const fn can_write(&self) -> bool {
        self.permission.can_write()
    }

    /// Returns the document's display name, delegating to the metadata
    /// (spec §6.10).
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
