//! Access rights and the persistent permission token for a document (spec §6.10).

use super::AccessMode;

#[doc = include_str!("../../docs/api/document/document_permission.doc/DocumentPermission.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentPermission {
    access: AccessMode,
    token: Option<Box<str>>,
}

impl DocumentPermission {
    /// Creates a read-only, ephemeral permission (spec §6.10).
    #[must_use]
    pub const fn read_only() -> Self {
        Self {
            access: AccessMode::Read,
            token: None,
        }
    }

    /// Creates a read-write, ephemeral permission (spec §6.10).
    #[must_use]
    pub const fn read_write() -> Self {
        Self {
            access: AccessMode::ReadWrite,
            token: None,
        }
    }

    #[doc = include_str!("../../docs/api/document/document_permission.doc/with_token.md")]
    #[must_use]
    pub fn with_token(mut self, token: impl Into<Box<str>>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Returns the granted access mode (spec §6.10).
    #[must_use]
    pub const fn access(&self) -> AccessMode {
        self.access
    }

    /// Returns whether this permission allows writing (spec §6.10).
    #[must_use]
    pub const fn can_write(&self) -> bool {
        self.access.can_write()
    }

    /// Returns the opaque persistent permission token, if any (spec §6.10).
    #[must_use]
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    #[doc = include_str!("../../docs/api/document/document_permission.doc/is_persistent.md")]
    #[must_use]
    pub const fn is_persistent(&self) -> bool {
        self.token.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_is_read_and_ephemeral() {
        let perm = DocumentPermission::read_only();
        assert_eq!(perm.access(), AccessMode::Read);
        assert!(!perm.can_write());
        assert_eq!(perm.token(), None);
        assert!(!perm.is_persistent());
    }

    #[test]
    fn read_write_permits_writing() {
        let perm = DocumentPermission::read_write();
        assert_eq!(perm.access(), AccessMode::ReadWrite);
        assert!(perm.can_write());
    }

    #[test]
    fn with_token_makes_it_persistent() {
        let perm = DocumentPermission::read_write().with_token("bookmark-7");
        assert_eq!(perm.token(), Some("bookmark-7"));
        assert!(perm.is_persistent());
    }
}
