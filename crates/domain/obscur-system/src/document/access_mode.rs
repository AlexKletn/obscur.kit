#[doc = include_str!("../../docs/api/document/document_permission.doc/AccessMode.md")]
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

#[cfg(test)]
mod tests {
    use super::AccessMode;

    #[test]
    fn access_mode_can_write() {
        assert!(!AccessMode::Read.can_write());
        assert!(AccessMode::ReadWrite.can_write());
    }
}
