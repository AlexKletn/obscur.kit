#![doc = include_str!("slot_rule.doc/_module.md")]

use obscur_view::contract::SlotCardinality;

#[doc = include_str!("slot_rule.doc/SlotRule.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRule {
    name: Box<str>,
    cardinality: SlotCardinality,
    allowed_children: Vec<Box<str>>,
}

impl SlotRule {
    #[doc = include_str!("slot_rule.doc/new.md")]
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, cardinality: SlotCardinality) -> Self {
        Self {
            name: name.into(),
            cardinality,
            allowed_children: Vec::new(),
        }
    }

    #[doc = include_str!("slot_rule.doc/allowing_child.md")]
    #[must_use]
    pub fn allowing_child(mut self, tag: impl Into<Box<str>>) -> Self {
        self.allowed_children.push(tag.into());
        self
    }

    #[doc = include_str!("slot_rule.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("slot_rule.doc/cardinality.md")]
    #[must_use]
    pub const fn cardinality(&self) -> SlotCardinality {
        self.cardinality
    }

    #[doc = include_str!("slot_rule.doc/allowed_children.md")]
    #[must_use]
    pub fn allowed_children(&self) -> &[Box<str>] {
        &self.allowed_children
    }

    #[doc = include_str!("slot_rule.doc/admits.md")]
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
