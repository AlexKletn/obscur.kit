//! `SlotRule`: one named slot in the compiler's string contract model (spec
//! §2.12).

use obscur_view::contract::SlotCardinality;

/// One named slot a component declares, in the string (tooling) contract model
/// the `.okui` compiler and the `okui!` macro both consult (spec §2.12).
///
/// A `.okui` source assigns a child to a slot by name — `<Title slot="header"/>`
/// — so the rule that checks the assignment must hold the name as written. The
/// symbol-keyed [`SlotContract`](obscur_view::contract::SlotContract) is the
/// runtime counterpart; see
/// [`SlotSurface`](obscur_view::contract::SlotSurface) for the same split at the
/// component-surface level.
///
/// [`cardinality`](Self::cardinality) is
/// [`SlotCardinality`](obscur_view::contract::SlotCardinality) itself, not a
/// re-spelling of it: the string and typed models disagree about *names*, which
/// is the whole reason both exist, but they must not disagree about how many
/// children a slot takes.
///
/// `allowed_children` is empty when the slot admits any child. A **component**'s
/// child surface is no longer a list like this one — it is a three-state
/// [`ChildPolicy`](crate::validation::ChildPolicy), because a component had to be
/// able to say it takes *no* children (§2.13) and an empty list already meant
/// "any". A slot needs no third state: a slot admitting nothing is a slot nothing
/// may be assigned to, which its [`cardinality`](Self::cardinality) states
/// directly.
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
