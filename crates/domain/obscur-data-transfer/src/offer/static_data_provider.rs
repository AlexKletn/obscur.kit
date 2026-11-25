use std::sync::Arc;

use crate::offer::MimeType;

use super::LazyDataProvider;

#[doc = include_str!("../../docs/api/offer/lazy_data_provider.doc/StaticDataProvider.md")]
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
        self.entries.iter().map(|(mime, _)| mime.clone()).collect()
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
