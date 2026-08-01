#![doc = include_str!("mime_type.doc/_module.md")]

#[doc = include_str!("mime_type.doc/MimeType.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MimeType {
    essence: Box<str>,
}

impl MimeType {
    #[doc = include_str!("mime_type.doc/new.md")]
    #[must_use]
    pub fn new(essence: impl Into<Box<str>>) -> Self {
        let essence = essence.into();
        Self {
            essence: essence.to_ascii_lowercase().into_boxed_str(),
        }
    }

    #[doc = include_str!("mime_type.doc/text_plain.md")]
    #[must_use]
    pub fn text_plain() -> Self {
        Self::new("text/plain")
    }

    #[doc = include_str!("mime_type.doc/text_html.md")]
    #[must_use]
    pub fn text_html() -> Self {
        Self::new("text/html")
    }

    #[doc = include_str!("mime_type.doc/text_uri_list.md")]
    #[must_use]
    pub fn text_uri_list() -> Self {
        Self::new("text/uri-list")
    }

    #[doc = include_str!("mime_type.doc/image_png.md")]
    #[must_use]
    pub fn image_png() -> Self {
        Self::new("image/png")
    }

    #[doc = include_str!("mime_type.doc/application_octet_stream.md")]
    #[must_use]
    pub fn application_octet_stream() -> Self {
        Self::new("application/octet-stream")
    }

    #[doc = include_str!("mime_type.doc/essence.md")]
    #[must_use]
    pub fn essence(&self) -> &str {
        &self.essence
    }

    #[doc = include_str!("mime_type.doc/is_text.md")]
    #[must_use]
    pub fn is_text(&self) -> bool {
        self.essence.starts_with("text/")
    }

    #[doc = include_str!("mime_type.doc/is_image.md")]
    #[must_use]
    pub fn is_image(&self) -> bool {
        self.essence.starts_with("image/")
    }

    #[doc = include_str!("mime_type.doc/matches.md")]
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
