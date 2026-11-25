//! A MIME descriptor for an offered data representation (spec §6.20 MIME
//! descriptors).

#[doc = include_str!("../../docs/api/offer/mime_type.doc/MimeType.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MimeType {
    essence: Box<str>,
}

impl MimeType {
    /// Creates a MIME descriptor from an `essence` string, normalizing it to
    /// lowercase (spec §6.20).
    #[must_use]
    pub fn new(essence: impl Into<Box<str>>) -> Self {
        let essence = essence.into();
        Self {
            essence: essence.to_ascii_lowercase().into_boxed_str(),
        }
    }

    /// The `text/plain` descriptor: plain text (spec §6.20).
    #[must_use]
    pub fn text_plain() -> Self {
        Self::new("text/plain")
    }

    /// The `text/html` descriptor: rich text (spec §6.20).
    #[must_use]
    pub fn text_html() -> Self {
        Self::new("text/html")
    }

    /// The `text/uri-list` descriptor: URIs / file references (spec §6.20).
    #[must_use]
    pub fn text_uri_list() -> Self {
        Self::new("text/uri-list")
    }

    /// The `image/png` descriptor: image data (spec §6.20).
    #[must_use]
    pub fn image_png() -> Self {
        Self::new("image/png")
    }

    /// The `application/octet-stream` descriptor: opaque bytes for files and
    /// custom formats (spec §6.20).
    #[must_use]
    pub fn application_octet_stream() -> Self {
        Self::new("application/octet-stream")
    }

    /// Returns the normalized `type/subtype` essence string (spec §6.20).
    #[must_use]
    pub fn essence(&self) -> &str {
        &self.essence
    }

    /// Returns whether this is a textual format (essence starts with `text/`)
    /// (spec §6.20).
    #[must_use]
    pub fn is_text(&self) -> bool {
        self.essence.starts_with("text/")
    }

    /// Returns whether this is an image format (essence starts with `image/`)
    /// (spec §6.20).
    #[must_use]
    pub fn is_image(&self) -> bool {
        self.essence.starts_with("image/")
    }

    /// Returns whether this descriptor names the same format as `other`, by
    /// essence equality (spec §6.20).
    #[must_use]
    pub fn matches(&self, other: &MimeType) -> bool {
        self.essence == other.essence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn common_constructors_have_expected_essences() {
        assert_eq!(MimeType::text_plain().essence(), "text/plain");
        assert_eq!(MimeType::text_html().essence(), "text/html");
        assert_eq!(MimeType::text_uri_list().essence(), "text/uri-list");
        assert_eq!(MimeType::image_png().essence(), "image/png");
        assert_eq!(
            MimeType::application_octet_stream().essence(),
            "application/octet-stream"
        );
    }

    #[test]
    fn is_text_and_is_image_classify_by_prefix() {
        assert!(MimeType::text_plain().is_text());
        assert!(MimeType::text_html().is_text());
        assert!(!MimeType::text_plain().is_image());
        assert!(MimeType::image_png().is_image());
        assert!(!MimeType::image_png().is_text());
        assert!(!MimeType::application_octet_stream().is_text());
        assert!(!MimeType::application_octet_stream().is_image());
    }

    #[test]
    fn matches_by_essence_equality() {
        assert!(MimeType::text_plain().matches(&MimeType::new("text/plain")));
        assert!(!MimeType::text_plain().matches(&MimeType::text_html()));
    }

    #[test]
    fn new_normalizes_to_lowercase() {
        let mime = MimeType::new("Text/HTML");
        assert_eq!(mime.essence(), "text/html");
        assert_eq!(mime, MimeType::text_html());
        assert!(mime.is_text());
    }

    #[test]
    fn equal_descriptors_dedup_in_hash_set() {
        let mut set = HashSet::new();
        set.insert(MimeType::new("IMAGE/PNG"));
        set.insert(MimeType::image_png());
        assert_eq!(set.len(), 1);
    }
}
