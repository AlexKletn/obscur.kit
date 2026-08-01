#![doc = include_str!("data_representation.doc/_module.md")]

use std::sync::Arc;

use crate::offer::MimeType;

#[doc = include_str!("data_representation.doc/DataRepresentation.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataRepresentation {
    mime: MimeType,
    inline: Option<Arc<[u8]>>,
}

impl DataRepresentation {
    #[doc = include_str!("data_representation.doc/eager.md")]
    #[must_use]
    pub fn eager(mime: MimeType, bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            mime,
            inline: Some(bytes.into()),
        }
    }

    #[doc = include_str!("data_representation.doc/lazy.md")]
    #[must_use]
    pub fn lazy(mime: MimeType) -> Self {
        Self { mime, inline: None }
    }

    #[doc = include_str!("data_representation.doc/mime.md")]
    #[must_use]
    pub fn mime(&self) -> &MimeType {
        &self.mime
    }

    #[doc = include_str!("data_representation.doc/inline_bytes.md")]
    #[must_use]
    pub fn inline_bytes(&self) -> Option<&[u8]> {
        self.inline.as_deref()
    }

    #[doc = include_str!("data_representation.doc/is_lazy.md")]
    #[must_use]
    pub fn is_lazy(&self) -> bool {
        self.inline.is_none()
    }

    #[doc = include_str!("data_representation.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> Option<usize> {
        self.inline.as_ref().map(|bytes| bytes.len())
    }

    #[doc = include_str!("data_representation.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == Some(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eager_exposes_inline_bytes_and_len() {
        let rep = DataRepresentation::eager(MimeType::text_plain(), b"hello".to_vec());
        assert_eq!(rep.mime(), &MimeType::text_plain());
        assert_eq!(rep.inline_bytes(), Some(&b"hello"[..]));
        assert_eq!(rep.len(), Some(5));
        assert!(!rep.is_lazy());
    }

    #[test]
    fn lazy_has_no_bytes_or_len() {
        let rep = DataRepresentation::lazy(MimeType::image_png());
        assert_eq!(rep.mime(), &MimeType::image_png());
        assert_eq!(rep.inline_bytes(), None);
        assert_eq!(rep.len(), None);
        assert!(rep.is_lazy());
    }

    #[test]
    fn equal_content_compares_equal() {
        let a = DataRepresentation::eager(MimeType::text_plain(), b"x".to_vec());
        let b = DataRepresentation::eager(MimeType::text_plain(), b"x".to_vec());
        assert_eq!(a, b);
    }
}
