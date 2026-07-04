//! What to open via the platform shell (spec §6.11 — shell integration).

use crate::document::DocumentHandle;

/// The typed activation category of a shell target (spec §6.11).
///
/// The toolkit surfaces typed activation events for files, URIs, and deep links
/// (spec §6.11); this enum names those three categories so an application can
/// route an incoming activation without re-parsing the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationKind {
    /// A file or resolved document handle.
    File,
    /// A conventional URI (an `http`, `https`, `mailto`, or `file` scheme).
    Uri,
    /// A custom/app-scheme URI used as a deep link into the application.
    DeepLink,
}

/// What to open through the platform shell (spec §6.11).
///
/// A [`ShellProvider`](crate::shell::ShellProvider) opens a target and reports a
/// [`ShellResult`](crate::shell::ShellResult). A target is a pure value naming a
/// file, an arbitrary URI, or an opaque document token — never a live handle to
/// the platform. File-association, MIME/protocol registration, and single-instance
/// concerns are the app/installer's responsibility and stay out of this type
/// (spec §6.11).
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

    /// Returns the typed activation category of this target (spec §6.11).
    ///
    /// A [`File`](Self::File) or [`Document`](Self::Document) activates as
    /// [`ActivationKind::File`]. A [`Uri`](Self::Uri) is classified by its scheme:
    /// the conventional `http`, `https`, `mailto`, and `file` schemes are
    /// [`ActivationKind::Uri`], and every other (custom/app) scheme is treated as
    /// a [`ActivationKind::DeepLink`]. This is the simplified §6.11 rule: rather
    /// than consult a registry, any non-standard scheme is assumed to be an app
    /// deep link.
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
