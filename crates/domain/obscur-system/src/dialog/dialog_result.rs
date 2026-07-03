//! The outcome of presenting a dialog (spec §6.9 — dialog providers).

use crate::document::{DirectoryHandle, DocumentHandle};

/// The outcome a [`DialogProvider`](crate::dialog::DialogProvider) returns for a
/// [`DialogRequest`](crate::dialog::DialogRequest) (spec §6.9).
///
/// The result is a pure value: each variant reports what the user chose, or that
/// the interaction ended without a choice. Successful file dialogs yield opaque
/// [`DocumentHandle`]s / [`DirectoryHandle`]s rather than paths (spec §6.10), so
/// the outcome carries no filesystem access of its own. [`Denied`](Self::Denied)
/// and [`Unsupported`](Self::Unsupported) let a caller fall back to another
/// provider in a chain.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogResult {
    /// The user selected zero or more files (an open result; possibly empty).
    Files(Vec<DocumentHandle>),
    /// The user chose a destination for a save request.
    Saved(DocumentHandle),
    /// The user selected a folder.
    Folder(DirectoryHandle),
    /// The user answered a message/confirmation — the button they chose.
    Confirmed(bool),
    /// The user closed the dialog without making a choice.
    Dismissed,
    /// The request was denied by permission policy.
    Denied,
    /// The provider does not support this kind of dialog.
    Unsupported,
}

impl DialogResult {
    /// Returns whether the user dismissed the dialog without choosing
    /// (spec §6.9).
    #[must_use]
    pub const fn is_dismissed(&self) -> bool {
        matches!(self, Self::Dismissed)
    }

    /// Returns whether the request was denied by permission policy (spec §6.9).
    #[must_use]
    pub const fn is_denied(&self) -> bool {
        matches!(self, Self::Denied)
    }

    /// Returns the selected files, or `None` when the result is not a
    /// [`Files`](Self::Files) outcome (spec §6.9).
    #[must_use]
    pub fn files(&self) -> Option<&[DocumentHandle]> {
        match self {
            Self::Files(files) => Some(files),
            _ => None,
        }
    }

    /// Returns the confirmation answer, or `None` when the result is not a
    /// [`Confirmed`](Self::Confirmed) outcome (spec §6.9).
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
