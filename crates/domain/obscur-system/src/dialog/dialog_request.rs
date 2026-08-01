#![doc = include_str!("dialog_request.doc/_module.md")]

#[doc = include_str!("dialog_request.doc/MessageLevel.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    #[doc = include_str!("dialog_request.doc/Info.md")]
    Info,
    #[doc = include_str!("dialog_request.doc/Warning.md")]
    Warning,
    #[doc = include_str!("dialog_request.doc/Error.md")]
    Error,
}

#[doc = include_str!("dialog_request.doc/FileFilter.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter {
    name: Box<str>,
    extensions: Vec<Box<str>>,
}

impl FileFilter {
    #[doc = include_str!("dialog_request.doc/new.md")]
    #[must_use]
    pub fn new(
        name: impl Into<Box<str>>,
        extensions: impl IntoIterator<Item = impl Into<Box<str>>>,
    ) -> Self {
        Self {
            name: name.into(),
            extensions: extensions.into_iter().map(Into::into).collect(),
        }
    }

    #[doc = include_str!("dialog_request.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("dialog_request.doc/extensions.md")]
    #[must_use]
    pub fn extensions(&self) -> &[Box<str>] {
        &self.extensions
    }
}

#[doc = include_str!("dialog_request.doc/DialogRequest.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum DialogRequest {
    #[doc = include_str!("dialog_request.doc/OpenFile.md")]
    OpenFile {
        #[doc = include_str!("dialog_request.doc/filters.md")]
        filters: Vec<FileFilter>,
        #[doc = include_str!("dialog_request.doc/allow_multiple.md")]
        allow_multiple: bool,
    },
    #[doc = include_str!("dialog_request.doc/SaveFile.md")]
    SaveFile {
        #[doc = include_str!("dialog_request.doc/suggested_name.md")]
        suggested_name: Box<str>,
        #[doc = include_str!("dialog_request.doc/filters_2.md")]
        filters: Vec<FileFilter>,
    },
    #[doc = include_str!("dialog_request.doc/PickFolder.md")]
    PickFolder,
    #[doc = include_str!("dialog_request.doc/Message.md")]
    Message {
        #[doc = include_str!("dialog_request.doc/title.md")]
        title: Box<str>,
        #[doc = include_str!("dialog_request.doc/body.md")]
        body: Box<str>,
        #[doc = include_str!("dialog_request.doc/level.md")]
        level: MessageLevel,
    },
    #[doc = include_str!("dialog_request.doc/Confirm.md")]
    Confirm {
        #[doc = include_str!("dialog_request.doc/title_2.md")]
        title: Box<str>,
        #[doc = include_str!("dialog_request.doc/body_2.md")]
        body: Box<str>,
    },
}

impl DialogRequest {
    #[doc = include_str!("dialog_request.doc/open_file.md")]
    #[must_use]
    pub const fn open_file() -> Self {
        Self::OpenFile {
            filters: Vec::new(),
            allow_multiple: false,
        }
    }

    #[doc = include_str!("dialog_request.doc/open_files.md")]
    #[must_use]
    pub const fn open_files() -> Self {
        Self::OpenFile {
            filters: Vec::new(),
            allow_multiple: true,
        }
    }

    #[doc = include_str!("dialog_request.doc/save_file.md")]
    #[must_use]
    pub fn save_file(suggested_name: impl Into<Box<str>>) -> Self {
        Self::SaveFile {
            suggested_name: suggested_name.into(),
            filters: Vec::new(),
        }
    }

    #[doc = include_str!("dialog_request.doc/pick_folder.md")]
    #[must_use]
    pub const fn pick_folder() -> Self {
        Self::PickFolder
    }

    #[doc = include_str!("dialog_request.doc/message.md")]
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

    #[doc = include_str!("dialog_request.doc/confirm.md")]
    #[must_use]
    pub fn confirm(title: impl Into<Box<str>>, body: impl Into<Box<str>>) -> Self {
        Self::Confirm {
            title: title.into(),
            body: body.into(),
        }
    }

    #[doc = include_str!("dialog_request.doc/is_file_dialog.md")]
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
    fn file_filter_stores_name_and_extensions() {
        let filter = FileFilter::new("Images", ["png", "jpg"]);
        assert_eq!(filter.name(), "Images");
        assert_eq!(filter.extensions(), &[Box::from("png"), Box::from("jpg")]);
    }

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
