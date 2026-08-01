#![doc = include_str!("dialog_result.doc/_module.md")]

use crate::document::{DirectoryHandle, DocumentHandle};

#[doc = include_str!("dialog_result.doc/DialogResult.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum DialogResult {
    #[doc = include_str!("dialog_result.doc/Files.md")]
    Files(Vec<DocumentHandle>),
    #[doc = include_str!("dialog_result.doc/Saved.md")]
    Saved(DocumentHandle),
    #[doc = include_str!("dialog_result.doc/Folder.md")]
    Folder(DirectoryHandle),
    #[doc = include_str!("dialog_result.doc/Confirmed.md")]
    Confirmed(bool),
    #[doc = include_str!("dialog_result.doc/Dismissed.md")]
    Dismissed,
    #[doc = include_str!("dialog_result.doc/Denied.md")]
    Denied,
    #[doc = include_str!("dialog_result.doc/Unsupported.md")]
    Unsupported,
}

impl DialogResult {
    #[doc = include_str!("dialog_result.doc/is_dismissed.md")]
    #[must_use]
    pub const fn is_dismissed(&self) -> bool {
        matches!(self, Self::Dismissed)
    }

    #[doc = include_str!("dialog_result.doc/is_denied.md")]
    #[must_use]
    pub const fn is_denied(&self) -> bool {
        matches!(self, Self::Denied)
    }

    #[doc = include_str!("dialog_result.doc/files.md")]
    #[must_use]
    pub fn files(&self) -> Option<&[DocumentHandle]> {
        match self {
            Self::Files(files) => Some(files),
            _ => None,
        }
    }

    #[doc = include_str!("dialog_result.doc/confirmed.md")]
    #[must_use]
    pub const fn confirmed(&self) -> Option<bool> {
        match self {
            Self::Confirmed(answer) => Some(*answer),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentMetadata, DocumentPermission};

    fn handle() -> DocumentHandle {
        DocumentHandle::new(
            "tok",
            DocumentMetadata::new("f"),
            DocumentPermission::read_only(),
        )
    }

    #[test]
    fn files_returns_slice_only_for_files_variant() {
        let result = DialogResult::Files(vec![handle()]);
        assert_eq!(result.files().map(<[_]>::len), Some(1));

        assert!(DialogResult::Dismissed.files().is_none());
        assert!(DialogResult::Files(Vec::new()).files() == Some(&[][..]));
    }

    #[test]
    fn confirmed_returns_answer_only_for_confirmed_variant() {
        assert_eq!(DialogResult::Confirmed(true).confirmed(), Some(true));
        assert_eq!(DialogResult::Confirmed(false).confirmed(), Some(false));
        assert_eq!(DialogResult::Dismissed.confirmed(), None);
    }

    #[test]
    fn dismissed_and_denied_predicates() {
        assert!(DialogResult::Dismissed.is_dismissed());
        assert!(!DialogResult::Denied.is_dismissed());
        assert!(DialogResult::Denied.is_denied());
        assert!(!DialogResult::Dismissed.is_denied());
        assert!(!DialogResult::Unsupported.is_denied());
    }
}
