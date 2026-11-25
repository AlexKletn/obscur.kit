//! A registered drop zone that accepts typed drags (spec §6.21 drag and drop —
//! typed target).

use crate::drag::DragEffect;
use crate::offer::{DataOffer, MimeType};
use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/drag/drop_target.doc/DropTarget.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct DropTarget {
    node: NodeId,
    accepted: Vec<MimeType>,
    allowed_effects: Vec<DragEffect>,
}

impl DropTarget {
    /// Creates a target on `node` that accepts any format and permits
    /// [`Copy`](DragEffect::Copy) and [`Move`](DragEffect::Move) (spec §6.21).
    #[must_use]
    pub fn new(node: NodeId) -> Self {
        Self {
            node,
            accepted: Vec::new(),
            allowed_effects: vec![DragEffect::Copy, DragEffect::Move],
        }
    }

    /// Returns this target with `mime` added to the accepted formats (spec
    /// §6.21).
    #[must_use]
    pub fn accepting(mut self, mime: MimeType) -> Self {
        self.accepted.push(mime);
        self
    }

    /// Returns this target with its permitted effects replaced by `effects`
    /// (spec §6.21).
    #[must_use]
    pub fn with_effects(mut self, effects: Vec<DragEffect>) -> Self {
        self.allowed_effects = effects;
        self
    }

    /// Returns the retained node that accepts drops (spec §6.21).
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    /// Returns the formats this target accepts; an empty slice means it accepts
    /// any format (spec §6.21).
    #[must_use]
    pub fn accepted(&self) -> &[MimeType] {
        &self.accepted
    }

    /// Returns the effects this target permits (spec §6.21).
    #[must_use]
    pub fn allowed_effects(&self) -> &[DragEffect] {
        &self.allowed_effects
    }

    #[doc = include_str!("../../docs/api/drag/drop_target.doc/accepts.md")]
    #[must_use]
    pub fn accepts(&self, offer: &DataOffer) -> bool {
        if self.accepted.is_empty() {
            return true;
        }
        let available = offer.available_mimes();
        self.accepted
            .iter()
            .any(|accepted| available.iter().any(|mime| accepted.matches(mime)))
    }

    /// Returns the effect a drop on this target would carry given a `preferred`
    /// choice, negotiated against the permitted effects (spec §6.21).
    #[must_use]
    pub fn effect_for(&self, preferred: DragEffect) -> DragEffect {
        DragEffect::first_allowed(&self.allowed_effects, preferred)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_any_and_permits_copy_move() {
        let target = DropTarget::new(NodeId::from_raw(1));
        assert_eq!(target.node(), NodeId::from_raw(1));
        assert!(target.accepted().is_empty());
        assert_eq!(
            target.allowed_effects(),
            &[DragEffect::Copy, DragEffect::Move]
        );
    }

    #[test]
    fn accepts_any_when_accepted_empty() {
        let target = DropTarget::new(NodeId::from_raw(1));
        assert!(target.accepts(&DataOffer::text("x")));
        assert!(target.accepts(&DataOffer::new()));
    }

    #[test]
    fn accepts_matching_mime() {
        let target = DropTarget::new(NodeId::from_raw(1)).accepting(MimeType::text_plain());
        assert!(target.accepts(&DataOffer::text("hi")));
    }

    #[test]
    fn rejects_non_matching_mime() {
        let target = DropTarget::new(NodeId::from_raw(1)).accepting(MimeType::image_png());
        assert!(!target.accepts(&DataOffer::text("hi")));
    }

    #[test]
    fn effect_for_negotiates_against_permitted() {
        let target = DropTarget::new(NodeId::from_raw(1));
        assert_eq!(target.effect_for(DragEffect::Move), DragEffect::Move);
        // Not permitted → first permitted.
        assert_eq!(target.effect_for(DragEffect::Link), DragEffect::Copy);
        // Link-only target with a rejected preference.
        let link_only = DropTarget::new(NodeId::from_raw(2)).with_effects(vec![DragEffect::Link]);
        assert_eq!(link_only.effect_for(DragEffect::Copy), DragEffect::Link);
    }
}
