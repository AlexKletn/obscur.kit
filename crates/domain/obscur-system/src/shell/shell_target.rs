//! What to open via the platform shell (spec §6.11 — shell integration).

use crate::document::DocumentHandle;

use super::ActivationKind;

#[doc = include_str!("../../docs/api/shell/shell_target.doc/ShellTarget.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ShellTarget {
    /// An opaque document handle to reveal or open (spec §6.10).
    File(DocumentHandle),
    /// An arbitrary URI to hand to the shell.
    Uri(Box<str>),
    /// An opaque document token to open through the shell.
    Document(Box<str>),
}

impl ShellTarget {
    /// Creates a target that opens the document behind `handle` (spec §6.11).
    #[must_use]
    pub const fn file(handle: DocumentHandle) -> Self {
        Self::File(handle)
    }

    /// Creates a target that opens the given `uri` (spec §6.11).
    #[must_use]
    pub fn uri(uri: impl Into<Box<str>>) -> Self {
        Self::Uri(uri.into())
    }

    /// Creates a target that opens the given opaque document `token`
    /// (spec §6.11).
    #[must_use]
    pub fn document(token: impl Into<Box<str>>) -> Self {
        Self::Document(token.into())
    }

    #[doc = include_str!("../../docs/api/shell/shell_target.doc/activation_kind.md")]
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

    /// Returns the URI string when this target is a [`Uri`](Self::Uri), else
    /// `None` (spec §6.11).
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
