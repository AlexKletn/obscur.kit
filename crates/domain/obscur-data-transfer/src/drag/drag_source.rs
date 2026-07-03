//! The origin of a drag and the data it carries (spec §6.21 drag and drop —
//! typed source).

use crate::drag::{DragEffect, DragPreview};
use crate::transfer::TransferPayload;
use obscur_identity::NodeId;

/// The origin of a drag, carrying the data being moved (spec §6.21 typed
/// source).
///
/// A drag source names the [`origin`](DragSource::origin) node the drag started
/// from, the [`payload`](DragSource::payload) it carries — which may expose both
/// an internal typed value and system MIME (spec §6.21) — the
/// [effects](DragEffect) the source permits, and an optional
/// [`preview`](DragSource::preview) image descriptor. A [`DragSession`] begins
/// from a source and negotiates its effect against each drop target.
///
/// [`DragSession`]: super::DragSession
#[derive(Debug, Clone, PartialEq)]
pub struct DragSource {
    payload: TransferPayload,
    allowed_effects: Vec<DragEffect>,
    preview: Option<DragPreview>,
    origin: NodeId,
}

impl DragSource {
    /// Creates a source dragging `payload` from `origin`, permitting
    /// [`Copy`](DragEffect::Copy) and [`Move`](DragEffect::Move) with no preview
    /// (spec §6.21).
    #[must_use]
    pub fn new(origin: NodeId, payload: TransferPayload) -> Self {
        Self {
            payload,
            allowed_effects: vec![DragEffect::Copy, DragEffect::Move],
            preview: None,
            origin,
        }
    }

    /// Returns this source with its permitted effects replaced by `effects`
    /// (spec §6.21).
    #[must_use]
    pub fn with_effects(mut self, effects: Vec<DragEffect>) -> Self {
        self.allowed_effects = effects;
        self
    }

    /// Returns this source with `preview` attached as its drag image (spec
    /// §6.21).
    #[must_use]
    pub fn with_preview(mut self, preview: DragPreview) -> Self {
        self.preview = Some(preview);
        self
    }

    /// Returns the payload being dragged (spec §6.21).
    #[must_use]
    pub fn payload(&self) -> &TransferPayload {
        &self.payload
    }

    /// Returns the effects this source permits (spec §6.21).
    #[must_use]
    pub fn allowed_effects(&self) -> &[DragEffect] {
        &self.allowed_effects
    }

    /// Returns the drag preview descriptor, if one was set (spec §6.21).
    #[must_use]
    pub fn preview(&self) -> Option<&DragPreview> {
        self.preview.as_ref()
    }

    /// Returns the node the drag started from (spec §6.21).
    #[must_use]
    pub const fn origin(&self) -> NodeId {
        self.origin
    }

    /// Returns whether the dragged payload carries nothing (spec §6.21).
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
