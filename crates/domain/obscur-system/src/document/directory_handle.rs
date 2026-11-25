//! An opaque handle to a directory/container (spec §6.10 — enumerated via the service).

use crate::document::{DocumentMetadata, DocumentPermission};

#[doc = include_str!("../../docs/api/document/directory_handle.doc/DirectoryHandle.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryHandle {
    token: Box<str>,
    metadata: DocumentMetadata,
    permission: DocumentPermission,
    path_hint: Option<Box<str>>,
}

impl DirectoryHandle {
    /// Creates a directory handle from its opaque token, metadata, and permission
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

    #[doc = include_str!("../../docs/api/document/directory_handle.doc/with_path_hint.md")]
    #[must_use]
    pub fn with_path_hint(mut self, path_hint: impl Into<Box<str>>) -> Self {
        self.path_hint = Some(path_hint.into());
        self
    }

    /// Returns the opaque host token identifying the directory (spec §6.10).
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

    /// Returns the directory's display name, delegating to the metadata
    /// (spec §6.10).
    #[must_use]
    pub fn name(&self) -> &str {
        self.metadata.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_and_accessors() {
        let dir = DirectoryHandle::new(
            "dir-1",
            DocumentMetadata::new("Photos").directory(true),
            DocumentPermission::read_only(),
        )
        .with_path_hint("/home/user/Photos");

        assert_eq!(dir.token(), "dir-1");
        assert_eq!(dir.name(), "Photos");
        assert!(dir.metadata().is_directory());
        assert_eq!(dir.permission(), &DocumentPermission::read_only());
        assert!(!dir.can_write());
        assert_eq!(dir.path_hint(), Some("/home/user/Photos"));
    }
}
