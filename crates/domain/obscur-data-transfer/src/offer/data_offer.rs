#![doc = include_str!("data_offer.doc/_module.md")]

use crate::offer::{DataRepresentation, MimeType};

#[doc = include_str!("data_offer.doc/DataOffer.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataOffer {
    representations: Vec<DataRepresentation>,
}

impl DataOffer {
    #[doc = include_str!("data_offer.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            representations: Vec::new(),
        }
    }

    #[doc = include_str!("data_offer.doc/single.md")]
    #[must_use]
    pub fn single(representation: DataRepresentation) -> Self {
        Self {
            representations: vec![representation],
        }
    }

    #[doc = include_str!("data_offer.doc/text.md")]
    #[must_use]
    pub fn text(text: impl Into<Box<str>>) -> Self {
        let text = text.into();
        Self::single(DataRepresentation::eager(
            MimeType::text_plain(),
            text.into_boxed_bytes(),
        ))
    }

    #[doc = include_str!("data_offer.doc/push.md")]
    pub fn push(&mut self, representation: DataRepresentation) {
        self.representations.push(representation);
    }

    #[doc = include_str!("data_offer.doc/with.md")]
    #[must_use]
    pub fn with(mut self, representation: DataRepresentation) -> Self {
        self.push(representation);
        self
    }

    #[doc = include_str!("data_offer.doc/representations.md")]
    #[must_use]
    pub fn representations(&self) -> &[DataRepresentation] {
        &self.representations
    }

    #[doc = include_str!("data_offer.doc/available_mimes.md")]
    #[must_use]
    pub fn available_mimes(&self) -> Vec<MimeType> {
        self.representations
            .iter()
            .map(|rep| rep.mime().clone())
            .collect()
    }

    #[doc = include_str!("data_offer.doc/has_mime.md")]
    #[must_use]
    pub fn has_mime(&self, mime: &MimeType) -> bool {
        self.representation_for(mime).is_some()
    }

    #[doc = include_str!("data_offer.doc/representation_for.md")]
    #[must_use]
    pub fn representation_for(&self, mime: &MimeType) -> Option<&DataRepresentation> {
        self.representations
            .iter()
            .find(|rep| rep.mime().matches(mime))
    }

    #[doc = include_str!("data_offer.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.representations.is_empty()
    }

    #[doc = include_str!("data_offer.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.representations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_builds_eager_plain_text_representation() {
        let offer = DataOffer::text("hi");
        assert_eq!(offer.len(), 1);
        let rep = offer
            .representation_for(&MimeType::text_plain())
            .expect("has text/plain");
        assert_eq!(rep.inline_bytes(), Some(&b"hi"[..]));
    }

    #[test]
    fn push_and_with_accumulate_representations() {
        let mut offer = DataOffer::new();
        assert!(offer.is_empty());
        offer.push(DataRepresentation::eager(
            MimeType::text_plain(),
            b"a".to_vec(),
        ));
        let offer = offer.with(DataRepresentation::lazy(MimeType::image_png()));
        assert_eq!(offer.len(), 2);
        assert!(!offer.is_empty());
    }

    #[test]
    fn has_mime_and_representation_for_match_by_essence() {
        let offer = DataOffer::single(DataRepresentation::lazy(MimeType::text_html()));
        assert!(offer.has_mime(&MimeType::new("TEXT/HTML")));
        assert!(!offer.has_mime(&MimeType::text_plain()));
        assert!(offer.representation_for(&MimeType::text_html()).is_some());
        assert!(offer.representation_for(&MimeType::image_png()).is_none());
    }

    #[test]
    fn available_mimes_preserves_order() {
        let offer = DataOffer::new()
            .with(DataRepresentation::lazy(MimeType::text_html()))
            .with(DataRepresentation::lazy(MimeType::text_plain()));
        assert_eq!(
            offer.available_mimes(),
            vec![MimeType::text_html(), MimeType::text_plain()]
        );
    }
}
