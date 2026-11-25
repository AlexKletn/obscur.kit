//! `SlotSurface`: the string-keyed *tooling* view of one declared slot (spec
//! §2.12).

use crate::contract::{ChildrenSurface, SlotCardinality};

#[doc = include_str!("../../docs/api/contract/slot_surface.doc/SlotSurface.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotSurface {
    name: &'static str,
    cardinality: SlotCardinality,
    children: ChildrenSurface,
}

impl SlotSurface {
    /// Declares a slot called `name` accepting `cardinality` children, with no
    /// restriction on which children those are.
    #[must_use]
    pub const fn new(name: &'static str, cardinality: SlotCardinality) -> Self {
        Self {
            name,
            cardinality,
            children: ChildrenSurface::Any,
        }
    }

    /// Adds `tag` to the slot's allowed child tags (fluent), switching the
    /// membership rule to [`ChildrenSurface::Only`].
    #[must_use]
    pub fn allowing_child(mut self, tag: &'static str) -> Self {
        match &mut self.children {
            ChildrenSurface::Only(tags) => tags.push(tag),
            _ => self.children = ChildrenSurface::Only(vec![tag]),
        }
        self
    }

    /// Returns the slot's name — the value a child's `slot="…"` must match.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns how many children the slot accepts.
    #[must_use]
    pub const fn cardinality(&self) -> SlotCardinality {
        self.cardinality
    }

    /// Returns which child tags the slot admits.
    #[must_use]
    pub const fn children(&self) -> &ChildrenSurface {
        &self.children
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_slot_is_unrestricted() {
        let slot = SlotSurface::new("body", SlotCardinality::Many);
        assert_eq!(slot.name(), "body");
        assert_eq!(slot.cardinality(), SlotCardinality::Many);
        assert_eq!(slot.children(), &ChildrenSurface::Any);
    }

    #[test]
    fn allowing_child_accumulates_a_tag_list() {
        let slot = SlotSurface::new("actions", SlotCardinality::Many)
            .allowing_child("Button")
            .allowing_child("Link");
        assert_eq!(
            slot.children(),
            &ChildrenSurface::Only(vec!["Button", "Link"])
        );
    }
}
