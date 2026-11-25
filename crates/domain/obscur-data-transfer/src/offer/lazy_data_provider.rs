#![doc = include_str!("../../docs/api/offer/lazy_data_provider.doc/_module.md")]

use std::sync::Arc;

use crate::offer::MimeType;

#[doc = include_str!("../../docs/api/offer/lazy_data_provider.doc/LazyDataProvider.md")]
pub trait LazyDataProvider {
    /// Resolves the bytes for `mime` on demand, or `None` if this provider
    /// cannot supply that format (spec §6.20).
    #[must_use]
    fn provide(&self, mime: &MimeType) -> Option<Arc<[u8]>>;

    /// Returns the MIME types this provider can resolve (spec §6.20).
    #[must_use]
    fn available_mimes(&self) -> Vec<MimeType>;
}
