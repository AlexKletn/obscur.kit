//! Access rights and the persistent permission token for a document (spec §6.10).

/// The access rights granted for a document handle (spec §6.10).
///
/// A file picker or portal grants either read-only or read-write access to the
/// document it returns; this enum records which. It carries no I/O and makes no
/// filesystem assumption — it is a pure statement of what the host permits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    /// The host permits reading the document but not modifying it.
    Read,
    /// The host permits both reading and writing the document.
    ReadWrite,
}

impl AccessMode {
    /// Returns whether this mode permits writing (spec §6.10).
    #[must_use]
    pub const fn can_write(&self) -> bool {
        matches!(self, Self::ReadWrite)
    }
}

/// The access rights and optional persistent permission token for a document
/// (spec §6.10 persistent permission token).
///
/// A document handle carries a permission describing what the host allows and,
/// optionally, an opaque **persistent permission token** the host can re-use to
/// re-acquire access to the same document in a later session (for example a
/// security-scoped bookmark on macOS or a portal document identifier on Linux).
/// A `None` token means access is ephemeral — valid only for the current session.
///
/// The token is opaque: this pure domain never interprets it, only stores and
/// hands it back. There is no filesystem path here (spec §6.10 — handles are not
/// paths).
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

    /// Returns this permission with its persistent permission token set
    /// (spec §6.10).
    ///
    /// The token is opaque host state that lets the host re-acquire access to the
    /// same document later; setting it makes the permission persistent.
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

    /// Returns whether this permission carries a persistent token (spec §6.10).
    ///
    /// A persistent permission can be re-used to re-acquire access in a later
    /// session; an ephemeral one is valid only for the current session.
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

    #[test]
    fn access_mode_can_write() {
        assert!(!AccessMode::Read.can_write());
        assert!(AccessMode::ReadWrite.can_write());
    }
}
