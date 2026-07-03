//! The port for resolving lazy representation bytes on demand (spec §6.20 lazy
//! async providers).
//!
//! This module defines [`LazyDataProvider`], the port through which a
//! [`DataOffer`](crate::offer::DataOffer)'s *lazy*
//! [representations](crate::offer::DataRepresentation) obtain their bytes when a
//! consumer finally asks for them. Interface adapters implement it — a clipboard
//! backend that reads the platform pasteboard, a drag source that serializes an
//! app value only if a drop actually lands. Lazy provision is inherently async
//! in real toolkits; the domain models it **synchronously**, so an async adapter
//! wraps its own future and calls this port when the bytes are ready. The
//! colocated [`StaticDataProvider`] is an honest reference implementation over
//! an in-memory table.

use std::sync::Arc;

use crate::offer::MimeType;

/// The port for resolving the bytes of a lazily-offered representation on demand
/// (spec §6.20 lazy async providers).
///
/// The trait is deliberately **object-safe** — no generic or `Self`-returning
/// methods — so a provider can be stored as `Box<dyn LazyDataProvider>` and held
/// polymorphically alongside an offer. A provider knows which MIME types it can
/// supply ([`Self::available_mimes`]) and resolves any one of them to bytes on
/// request ([`Self::provide`]), returning `None` when it cannot.
pub trait LazyDataProvider {
    /// Resolves the bytes for `mime` on demand, or `None` if this provider
    /// cannot supply that format (spec §6.20).
    #[must_use]
    fn provide(&self, mime: &MimeType) -> Option<Arc<[u8]>>;

    /// Returns the MIME types this provider can resolve (spec §6.20).
    #[must_use]
    fn available_mimes(&self) -> Vec<MimeType>;
}

/// A deterministic in-memory [`LazyDataProvider`] reference implementation (spec
/// §6.20).
///
/// `StaticDataProvider` holds an ordered table of `(MimeType, bytes)` entries so
/// that [`Self::available_mimes`] reports them in insertion order and lookups are
/// reproducible. It is useful as a test double and as the honest provider for
/// offers whose lazy bytes are in fact already known. Entries are matched by
/// [`MimeType::matches`], so lookup is by format essence.
#[derive(Debug, Default, Clone)]
pub struct StaticDataProvider {
    entries: Vec<(MimeType, Arc<[u8]>)>,
}

impl StaticDataProvider {
    /// Creates an empty provider (spec §6.20).
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Returns this provider with an entry mapping `mime` to `bytes` appended
    /// (spec §6.20).
    #[must_use]
    pub fn with(mut self, mime: MimeType, bytes: impl Into<Arc<[u8]>>) -> Self {
        self.entries.push((mime, bytes.into()));
        self
    }
}

impl LazyDataProvider for StaticDataProvider {
    fn provide(&self, mime: &MimeType) -> Option<Arc<[u8]>> {
        self.entries
            .iter()
            .find(|(entry_mime, _)| entry_mime.matches(mime))
            .map(|(_, bytes)| Arc::clone(bytes))
    }

    fn available_mimes(&self) -> Vec<MimeType> {
        self.entries
            .iter()
            .map(|(mime, _)| mime.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provide_resolves_known_mime_by_match() {
        let provider = StaticDataProvider::new()
            .with(MimeType::text_plain(), b"hi".to_vec())
            .with(MimeType::image_png(), vec![1u8, 2, 3]);
        assert_eq!(
            provider.provide(&MimeType::new("TEXT/PLAIN")).as_deref(),
            Some(&b"hi"[..])
        );
        assert_eq!(
            provider.provide(&MimeType::image_png()).as_deref(),
            Some(&[1u8, 2, 3][..])
        );
    }

    #[test]
    fn provide_returns_none_for_unknown_mime() {
        let provider = StaticDataProvider::new().with(MimeType::text_plain(), b"hi".to_vec());
        assert!(provider.provide(&MimeType::text_html()).is_none());
    }

    #[test]
    fn available_mimes_lists_in_insertion_order() {
        let provider = StaticDataProvider::new()
            .with(MimeType::image_png(), vec![0u8])
            .with(MimeType::text_plain(), b"a".to_vec());
        assert_eq!(
            provider.available_mimes(),
            vec![MimeType::image_png(), MimeType::text_plain()]
        );
    }

    #[test]
    fn provider_is_object_safe() {
        let provider: Box<dyn LazyDataProvider> = Box::new(StaticDataProvider::new());
        assert!(provider.available_mimes().is_empty());
    }
}
