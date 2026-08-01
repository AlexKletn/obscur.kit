#![doc = include_str!("transfer_payload.doc/_module.md")]

use crate::offer::DataOffer;

#[doc = include_str!("transfer_payload.doc/TransferPayload.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferPayload {
    offer: DataOffer,
    internal_type: Option<Box<str>>,
    internal_data: Option<Box<str>>,
}

impl TransferPayload {
    #[doc = include_str!("transfer_payload.doc/new.md")]
    #[must_use]
    pub fn new(offer: DataOffer) -> Self {
        Self {
            offer,
            internal_type: None,
            internal_data: None,
        }
    }

    #[doc = include_str!("transfer_payload.doc/with_internal.md")]
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

    #[doc = include_str!("transfer_payload.doc/offer.md")]
    #[must_use]
    pub fn offer(&self) -> &DataOffer {
        &self.offer
    }

    #[doc = include_str!("transfer_payload.doc/internal_type.md")]
    #[must_use]
    pub fn internal_type(&self) -> Option<&str> {
        self.internal_type.as_deref()
    }

    #[doc = include_str!("transfer_payload.doc/internal_data.md")]
    #[must_use]
    pub fn internal_data(&self) -> Option<&str> {
        self.internal_data.as_deref()
    }

    #[doc = include_str!("transfer_payload.doc/has_internal.md")]
    #[must_use]
    pub fn has_internal(&self) -> bool {
        self.internal_type.is_some()
    }

    #[doc = include_str!("transfer_payload.doc/is_empty.md")]
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
