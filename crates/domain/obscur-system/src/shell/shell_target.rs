#![doc = include_str!("shell_target.doc/_module.md")]

use crate::document::DocumentHandle;

#[doc = include_str!("shell_target.doc/ActivationKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationKind {
    #[doc = include_str!("shell_target.doc/File.md")]
    File,
    #[doc = include_str!("shell_target.doc/Uri.md")]
    Uri,
    #[doc = include_str!("shell_target.doc/DeepLink.md")]
    DeepLink,
}

#[doc = include_str!("shell_target.doc/ShellTarget.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ShellTarget {
    #[doc = include_str!("shell_target.doc/File_2.md")]
    File(DocumentHandle),
    #[doc = include_str!("shell_target.doc/Uri_2.md")]
    Uri(Box<str>),
    #[doc = include_str!("shell_target.doc/Document.md")]
    Document(Box<str>),
}

impl ShellTarget {
    #[doc = include_str!("shell_target.doc/file.md")]
    #[must_use]
    pub const fn file(handle: DocumentHandle) -> Self {
        Self::File(handle)
    }

    #[doc = include_str!("shell_target.doc/uri.md")]
    #[must_use]
    pub fn uri(uri: impl Into<Box<str>>) -> Self {
        Self::Uri(uri.into())
    }

    #[doc = include_str!("shell_target.doc/document.md")]
    #[must_use]
    pub fn document(token: impl Into<Box<str>>) -> Self {
        Self::Document(token.into())
    }

    #[doc = include_str!("shell_target.doc/activation_kind.md")]
    #[must_use]
    pub fn activation_kind(&self) -> ActivationKind {
        match self {
            Self::File(_) | Self::Document(_) => ActivationKind::File,
            Self::Uri(uri) => {
                let scheme = uri.split_once(':').map(|(scheme, _)| scheme);
                match scheme {
                    Some("http" | "https" | "mailto" | "file") => ActivationKind::Uri,
                    _ => ActivationKind::DeepLink,
                }
            }
        }
    }

    #[doc = include_str!("shell_target.doc/as_uri.md")]
    #[must_use]
    pub fn as_uri(&self) -> Option<&str> {
        match self {
            Self::Uri(uri) => Some(uri),
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
    fn file_and_document_activate_as_file() {
        assert_eq!(
            ShellTarget::file(handle()).activation_kind(),
            ActivationKind::File
        );
        assert_eq!(
            ShellTarget::document("doc-token").activation_kind(),
            ActivationKind::File
        );
    }

    #[test]
    fn standard_scheme_uri_activates_as_uri() {
        assert_eq!(
            ShellTarget::uri("https://example.com").activation_kind(),
            ActivationKind::Uri
        );
        assert_eq!(
            ShellTarget::uri("mailto:a@b.com").activation_kind(),
            ActivationKind::Uri
        );
    }

    #[test]
    fn custom_scheme_uri_activates_as_deep_link() {
        assert_eq!(
            ShellTarget::uri("myapp://open/thing").activation_kind(),
            ActivationKind::DeepLink
        );
        // No scheme at all is likewise treated as a deep link.
        assert_eq!(
            ShellTarget::uri("bareword").activation_kind(),
            ActivationKind::DeepLink
        );
    }

    #[test]
    fn as_uri_returns_only_for_uri_targets() {
        assert_eq!(ShellTarget::uri("https://x").as_uri(), Some("https://x"));
        assert_eq!(ShellTarget::document("d").as_uri(), None);
        assert_eq!(ShellTarget::file(handle()).as_uri(), None);
    }
}
