#![doc = include_str!("document_source.doc/_module.md")]

#[doc = include_str!("document_source.doc/DocumentSource.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentSource {
    token: Box<str>,
    path_hint: Option<Box<str>>,
}

impl DocumentSource {
    #[doc = include_str!("document_source.doc/new.md")]
    #[must_use]
    pub fn new(token: impl Into<Box<str>>) -> Self {
        Self {
            token: token.into(),
            path_hint: None,
        }
    }

    #[doc = include_str!("document_source.doc/with_path_hint.md")]
    #[must_use]
    pub fn with_path_hint(mut self, path_hint: impl Into<Box<str>>) -> Self {
        self.path_hint = Some(path_hint.into());
        self
    }

    #[doc = include_str!("document_source.doc/token.md")]
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    #[doc = include_str!("document_source.doc/path_hint.md")]
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
