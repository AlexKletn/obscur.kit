//! `SlotRule`: one named slot in the compiler's string contract model (spec
//! §2.12).

use obscur_view::contract::SlotCardinality;

#[doc = include_str!("../../docs/api/validation/slot_rule.doc/SlotRule.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRule {
    name: Box<str>,
    cardinality: SlotCardinality,
    allowed_children: Vec<Box<str>>,
}

impl SlotRule {
    /// Declares a slot called `name` accepting `cardinality` children, with no
    /// restriction on which children those are.
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, cardinality: SlotCardinality) -> Self {
        Self {
            name: name.into(),
            cardinality,
            allowed_children: Vec::new(),
        }
    }

    /// Adds `tag` to the slot's allowed child tags (fluent). Adding any turns on
    /// the restriction: only listed tags are then accepted in this slot.
    #[must_use]
    pub fn allowing_child(mut self, tag: impl Into<Box<str>>) -> Self {
        self.allowed_children.push(tag.into());
        self
    }

    /// Returns the slot's name — the value a child's `slot="…"` must match.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns how many children the slot accepts.
    #[must_use]
    pub const fn cardinality(&self) -> SlotCardinality {
        self.cardinality
    }

    /// Returns the child tags the slot admits; an empty slice means any child.
    #[must_use]
    pub fn allowed_children(&self) -> &[Box<str>] {
        &self.allowed_children
    }

    /// Returns whether `tag` may be placed in this slot.
    pub(crate) fn admits(&self, tag: &str) -> bool {
        self.allowed_children.is_empty() || self.allowed_children.iter().any(|t| t.as_ref() == tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_slot_admits_any_child() {
        let slot = SlotRule::new("body", SlotCardinality::Many);
        assert_eq!(slot.name(), "body");
        assert_eq!(slot.cardinality(), SlotCardinality::Many);
        assert!(slot.allowed_children().is_empty());
        assert!(slot.admits("anything"));
    }

    #[test]
    fn a_restricted_slot_admits_only_its_listed_tags() {
        let slot = SlotRule::new("actions", SlotCardinality::Many).allowing_child("Button");
        assert!(slot.admits("Button"));
        assert!(!slot.admits("Input"));
    }
}
