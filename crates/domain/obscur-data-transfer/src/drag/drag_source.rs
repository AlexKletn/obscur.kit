#![doc = include_str!("drag_source.doc/_module.md")]

use crate::drag::{DragEffect, DragPreview};
use crate::transfer::TransferPayload;
use obscur_identity::NodeId;

#[doc = include_str!("drag_source.doc/DragSource.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct DragSource {
    payload: TransferPayload,
    allowed_effects: Vec<DragEffect>,
    preview: Option<DragPreview>,
    origin: NodeId,
}

impl DragSource {
    #[doc = include_str!("drag_source.doc/new.md")]
    #[must_use]
    pub fn new(origin: NodeId, payload: TransferPayload) -> Self {
        Self {
            payload,
            allowed_effects: vec![DragEffect::Copy, DragEffect::Move],
            preview: None,
            origin,
        }
    }

    #[doc = include_str!("drag_source.doc/with_effects.md")]
    #[must_use]
    pub fn with_effects(mut self, effects: Vec<DragEffect>) -> Self {
        self.allowed_effects = effects;
        self
    }

    #[doc = include_str!("drag_source.doc/with_preview.md")]
    #[must_use]
    pub fn with_preview(mut self, preview: DragPreview) -> Self {
        self.preview = Some(preview);
        self
    }

    #[doc = include_str!("drag_source.doc/payload.md")]
    #[must_use]
    pub fn payload(&self) -> &TransferPayload {
        &self.payload
    }

    #[doc = include_str!("drag_source.doc/allowed_effects.md")]
    #[must_use]
    pub fn allowed_effects(&self) -> &[DragEffect] {
        &self.allowed_effects
    }

    #[doc = include_str!("drag_source.doc/preview.md")]
    #[must_use]
    pub fn preview(&self) -> Option<&DragPreview> {
        self.preview.as_ref()
    }

    #[doc = include_str!("drag_source.doc/origin.md")]
    #[must_use]
    pub const fn origin(&self) -> NodeId {
        self.origin
    }

    #[doc = include_str!("drag_source.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offer::DataOffer;
    use obscur_geometry::Size;

    fn payload() -> TransferPayload {
        TransferPayload::new(DataOffer::text("x"))
    }

    #[test]
    fn new_permits_copy_move_with_no_preview() {
        let source = DragSource::new(NodeId::from_raw(2), payload());
        assert_eq!(source.origin(), NodeId::from_raw(2));
        assert_eq!(
            source.allowed_effects(),
            &[DragEffect::Copy, DragEffect::Move]
        );
        assert!(source.preview().is_none());
        assert!(!source.is_empty());
    }

    #[test]
    fn builder_sets_effects_and_preview() {
        let preview = DragPreview::new(Size::new(32.0, 32.0));
        let source = DragSource::new(NodeId::from_raw(2), payload())
            .with_effects(vec![DragEffect::Link])
            .with_preview(preview);
        assert_eq!(source.allowed_effects(), &[DragEffect::Link]);
        assert_eq!(source.preview(), Some(&preview));
    }

    #[test]
    fn is_empty_reflects_empty_payload() {
        let source = DragSource::new(NodeId::from_raw(2), TransferPayload::new(DataOffer::new()));
        assert!(source.is_empty());
    }
}
