//! A request to present a dialog (spec §6.9 — dialog providers).

use super::{FileFilter, MessageLevel};

#[doc = include_str!("../../docs/api/dialog/dialog_request.doc/DialogRequest.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum DialogRequest {
    /// Pick one or more existing files, optionally constrained by `filters`.
    OpenFile {
        /// The file-type filters offered to the user; empty means unfiltered.
        filters: Vec<FileFilter>,
        /// Whether the user may select more than one file.
        allow_multiple: bool,
    },
    /// Choose a destination to save a file to.
    SaveFile {
        /// The default file name proposed to the user.
        suggested_name: Box<str>,
        /// The file-type filters offered to the user; empty means unfiltered.
        filters: Vec<FileFilter>,
    },
    /// Pick a single folder / directory.
    PickFolder,
    /// Show an informational message the user acknowledges.
    Message {
        /// The dialog's title line.
        title: Box<str>,
        /// The dialog's body text.
        body: Box<str>,
        /// The severity/intent of the message.
        level: MessageLevel,
    },
    /// Ask the user to confirm or cancel an action.
    Confirm {
        /// The dialog's title line.
        title: Box<str>,
        /// The dialog's body text describing what is being confirmed.
        body: Box<str>,
    },
}

impl DialogRequest {
    /// Creates a single-file open request with no filters (spec §6.9).
    #[must_use]
    pub const fn open_file() -> Self {
        Self::OpenFile {
            filters: Vec::new(),
            allow_multiple: false,
        }
    }

    /// Creates a multi-file open request with no filters (spec §6.9).
    #[must_use]
    pub const fn open_files() -> Self {
        Self::OpenFile {
            filters: Vec::new(),
            allow_multiple: true,
        }
    }

    /// Creates a save request proposing `suggested_name`, with no filters
    /// (spec §6.9).
    #[must_use]
    pub fn save_file(suggested_name: impl Into<Box<str>>) -> Self {
        Self::SaveFile {
            suggested_name: suggested_name.into(),
            filters: Vec::new(),
        }
    }

    /// Creates a folder-picker request (spec §6.9).
    #[must_use]
    pub const fn pick_folder() -> Self {
        Self::PickFolder
    }

    /// Creates a message request with the given `title`, `body`, and `level`
    /// (spec §6.9).
    #[must_use]
    pub fn message(
        title: impl Into<Box<str>>,
        body: impl Into<Box<str>>,
        level: MessageLevel,
    ) -> Self {
        Self::Message {
            title: title.into(),
            body: body.into(),
            level,
        }
    }

    /// Creates a confirmation request with the given `title` and `body`
    /// (spec §6.9).
    #[must_use]
    pub fn confirm(title: impl Into<Box<str>>, body: impl Into<Box<str>>) -> Self {
        Self::Confirm {
            title: title.into(),
            body: body.into(),
        }
    }

    /// Returns whether this request is a file/folder dialog rather than a
    /// message or confirmation (spec §6.9).
    #[must_use]
    pub const fn is_file_dialog(&self) -> bool {
        matches!(
            self,
            Self::OpenFile { .. } | Self::SaveFile { .. } | Self::PickFolder
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_file_constructors_set_multiplicity() {
        assert_eq!(
            DialogRequest::open_file(),
            DialogRequest::OpenFile {
                filters: Vec::new(),
                allow_multiple: false,
            }
        );
        assert_eq!(
            DialogRequest::open_files(),
            DialogRequest::OpenFile {
                filters: Vec::new(),
                allow_multiple: true,
            }
        );
    }

    #[test]
    fn save_message_confirm_and_folder_constructors() {
        assert_eq!(
            DialogRequest::save_file("draft.txt"),
            DialogRequest::SaveFile {
                suggested_name: Box::from("draft.txt"),
                filters: Vec::new(),
            }
        );
        assert_eq!(DialogRequest::pick_folder(), DialogRequest::PickFolder);
        assert_eq!(
            DialogRequest::message("Hi", "Body", MessageLevel::Warning),
            DialogRequest::Message {
                title: Box::from("Hi"),
                body: Box::from("Body"),
                level: MessageLevel::Warning,
            }
        );
        assert_eq!(
            DialogRequest::confirm("Delete?", "Are you sure?"),
            DialogRequest::Confirm {
                title: Box::from("Delete?"),
                body: Box::from("Are you sure?"),
            }
        );
    }

    #[test]
    fn is_file_dialog_distinguishes_file_from_message_requests() {
        assert!(DialogRequest::open_file().is_file_dialog());
        assert!(DialogRequest::open_files().is_file_dialog());
        assert!(DialogRequest::save_file("a").is_file_dialog());
        assert!(DialogRequest::pick_folder().is_file_dialog());
        assert!(!DialogRequest::message("t", "b", MessageLevel::Info).is_file_dialog());
        assert!(!DialogRequest::confirm("t", "b").is_file_dialog());
    }
}
