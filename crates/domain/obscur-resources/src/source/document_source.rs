//! A resource backed by a host document handle (spec §5.16, §6.10).

/// A resource backed by a document the host resolves (spec §5.16 `DocumentHandle`,
/// §6.10 file pickers).
///
/// The concrete `DocumentHandle` type lives in the platform layer and is not a
/// dependency of this crate, so a document source models it abstractly: an opaque
/// `token` string the host resolves back to its document, plus an optional
/// `path_hint`.
///
/// The kit does not read the document itself here — this is a pure description.
/// A loader or host resolves the token to actual bytes later (spec §5.15: the kit
/// performs no arbitrary I/O by itself).
///
/// The `path_hint` is *not* necessarily a real filesystem path (spec §6.10): a
/// file picker may hand back only a display name or a sandboxed identifier, so the
/// hint is advisory metadata for display or logging, never something to open.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentSource {
    token: Box<str>,
    path_hint: Option<Box<str>>,
}

impl DocumentSource {
    /// Creates a document source from an opaque host handle token.
    #[must_use]
    pub fn new(token: impl Into<Box<str>>) -> Self {
        Self {
            token: token.into(),
            path_hint: None,
        }
    }

    /// Returns this source with its advisory path hint set (spec §6.10).
    ///
    /// The hint is not necessarily a real filesystem path; see the type docs.
    #[must_use]
    pub fn with_path_hint(mut self, path_hint: impl Into<Box<str>>) -> Self {
        self.path_hint = Some(path_hint.into());
        self
    }

    /// Returns the opaque handle token the host resolves (spec §5.16).
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns the advisory path hint, if any (spec §6.10).
    #[must_use]
    pub fn path_hint(&self) -> Option<&str> {
        self.path_hint.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_expose_token() {
        let src = DocumentSource::new("handle-42");
        assert_eq!(src.token(), "handle-42");
        assert_eq!(src.path_hint(), None);
    }

    #[test]
    fn with_path_hint_sets_hint() {
        let src = DocumentSource::new("handle-42").with_path_hint("photo.png");
        assert_eq!(src.path_hint(), Some("photo.png"));
    }

    #[test]
    fn equal_tokens_and_hints_are_equal() {
        let a = DocumentSource::new("h").with_path_hint("p");
        let b = DocumentSource::new("h").with_path_hint("p");
        assert_eq!(a, b);
    }

    #[test]
    fn different_token_or_hint_distinct() {
        let a = DocumentSource::new("h");
        let b = DocumentSource::new("h").with_path_hint("p");
        let c = DocumentSource::new("other");
        assert_ne!(a, b);
        assert_ne!(a, c);
    }
}
