//! The transferable content: a MIME offer plus an optional typed internal
//! payload (spec §6.20 typed internal payload, §6.21).

use crate::offer::DataOffer;

#[doc = include_str!("../../docs/api/transfer/transfer_payload.doc/TransferPayload.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferPayload {
    offer: DataOffer,
    internal_type: Option<Box<str>>,
    internal_data: Option<Box<str>>,
}

impl TransferPayload {
    /// Creates a payload carrying only the system MIME `offer` (spec §6.20).
    #[must_use]
    pub fn new(offer: DataOffer) -> Self {
        Self {
            offer,
            internal_type: None,
            internal_data: None,
        }
    }

    #[doc = include_str!("../../docs/api/transfer/transfer_payload.doc/with_internal.md")]
    #[must_use]
    pub fn with_internal(
        mut self,
        type_tag: impl Into<Box<str>>,
        data: impl Into<Box<str>>,
    ) -> Self {
        self.internal_type = Some(type_tag.into());
        self.internal_data = Some(data.into());
        self
    }

    /// Returns the system MIME offer (spec §6.20).
    #[must_use]
    pub fn offer(&self) -> &DataOffer {
        &self.offer
    }

    /// Returns the internal type tag, if a typed internal path is present (spec
    /// §6.21).
    #[must_use]
    pub fn internal_type(&self) -> Option<&str> {
        self.internal_type.as_deref()
    }

    /// Returns the serialized internal data, if a typed internal path is present
    /// (spec §6.21).
    #[must_use]
    pub fn internal_data(&self) -> Option<&str> {
        self.internal_data.as_deref()
    }

    /// Returns whether a typed internal path is present (spec §6.21).
    #[must_use]
    pub fn has_internal(&self) -> bool {
        self.internal_type.is_some()
    }

    /// Returns whether the payload carries nothing — an empty offer and no
    /// internal path (spec §6.20).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.offer.is_empty() && !self.has_internal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offer::{DataRepresentation, MimeType};

    #[test]
    fn new_has_no_internal_path() {
        let payload = TransferPayload::new(DataOffer::text("hi"));
        assert!(!payload.has_internal());
        assert_eq!(payload.internal_type(), None);
        assert_eq!(payload.internal_data(), None);
    }

    #[test]
    fn with_internal_attaches_typed_path() {
        let payload =
            TransferPayload::new(DataOffer::text("hi")).with_internal("app/node", "{\"id\":7}");
        assert!(payload.has_internal());
        assert_eq!(payload.internal_type(), Some("app/node"));
        assert_eq!(payload.internal_data(), Some("{\"id\":7}"));
    }

    #[test]
    fn offer_passes_through() {
        let offer = DataOffer::single(DataRepresentation::lazy(MimeType::image_png()));
        let payload = TransferPayload::new(offer.clone());
        assert_eq!(payload.offer(), &offer);
    }

    #[test]
    fn is_empty_only_when_offer_empty_and_no_internal() {
        assert!(TransferPayload::new(DataOffer::new()).is_empty());
        assert!(!TransferPayload::new(DataOffer::text("x")).is_empty());
        assert!(
            !TransferPayload::new(DataOffer::new())
                .with_internal("t", "d")
                .is_empty()
        );
    }
}
