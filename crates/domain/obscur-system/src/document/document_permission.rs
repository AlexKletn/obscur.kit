#![doc = include_str!("document_permission.doc/_module.md")]

#[doc = include_str!("document_permission.doc/AccessMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    #[doc = include_str!("document_permission.doc/Read.md")]
    Read,
    #[doc = include_str!("document_permission.doc/ReadWrite.md")]
    ReadWrite,
}

impl AccessMode {
    #[doc = include_str!("document_permission.doc/can_write.md")]
    #[must_use]
    pub const fn can_write(&self) -> bool {
        matches!(self, Self::ReadWrite)
    }
}

#[doc = include_str!("document_permission.doc/DocumentPermission.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentPermission {
    access: AccessMode,
    token: Option<Box<str>>,
}

impl DocumentPermission {
    #[doc = include_str!("document_permission.doc/read_only.md")]
    #[must_use]
    pub const fn read_only() -> Self {
        Self {
            access: AccessMode::Read,
            token: None,
        }
    }

    #[doc = include_str!("document_permission.doc/read_write.md")]
    #[must_use]
    pub const fn read_write() -> Self {
        Self {
            access: AccessMode::ReadWrite,
            token: None,
        }
    }

    #[doc = include_str!("document_permission.doc/with_token.md")]
    #[must_use]
    pub fn with_token(mut self, token: impl Into<Box<str>>) -> Self {
        self.token = Some(token.into());
        self
    }

    #[doc = include_str!("document_permission.doc/access.md")]
    #[must_use]
    pub const fn access(&self) -> AccessMode {
        self.access
    }

    #[doc = include_str!("document_permission.doc/can_write_2.md")]
    #[must_use]
    pub const fn can_write(&self) -> bool {
        self.access.can_write()
    }

    #[doc = include_str!("document_permission.doc/token.md")]
    #[must_use]
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    #[doc = include_str!("document_permission.doc/is_persistent.md")]
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
