//! One representation of offered data in a specific MIME format (spec §6.20
//! multiple representations).

use std::sync::Arc;

use crate::offer::MimeType;

/// A single representation of a [`DataOffer`](crate::offer::DataOffer)'s content
/// in one MIME format (spec §6.20).
///
/// An offer typically advertises the same logical content under several MIME
/// types; each is one `DataRepresentation`. A representation is either *eager* —
/// its bytes are already in hand, held in an [`Arc<[u8]>`] so clones are cheap
/// and equal content compares equal — or *lazy*, meaning the bytes are not
/// materialized yet and must be fetched on demand from a
/// [`LazyDataProvider`](crate::offer::LazyDataProvider). Modeling async
/// provision is an adapter concern; this value only records *which* MIME the
/// representation carries and whether its bytes are inline.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataRepresentation {
    mime: MimeType,
    inline: Option<Arc<[u8]>>,
}

impl DataRepresentation {
    /// Creates an eager representation whose `bytes` are already available (spec
    /// §6.20).
    #[must_use]
    pub fn eager(mime: MimeType, bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            mime,
            inline: Some(bytes.into()),
        }
    }

    /// Creates a lazy representation whose bytes must be fetched on demand (spec
    /// §6.20).
    #[must_use]
    pub fn lazy(mime: MimeType) -> Self {
        Self { mime, inline: None }
    }

    /// Returns the MIME descriptor for this representation (spec §6.20).
    #[must_use]
    pub fn mime(&self) -> &MimeType {
        &self.mime
    }

    /// Returns the inline bytes if this representation is eager, otherwise
    /// `None` (spec §6.20).
    #[must_use]
    pub fn inline_bytes(&self) -> Option<&[u8]> {
        self.inline.as_deref()
    }

    /// Returns whether this representation is lazy (its bytes are not inline)
    /// (spec §6.20).
    #[must_use]
    pub fn is_lazy(&self) -> bool {
        self.inline.is_none()
    }

    /// Returns the eager byte length, or `None` for a lazy representation (spec
    /// §6.20).
    #[must_use]
    pub fn len(&self) -> Option<usize> {
        self.inline.as_ref().map(|bytes| bytes.len())
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
