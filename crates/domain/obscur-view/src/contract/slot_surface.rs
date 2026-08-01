#![doc = include_str!("slot_surface.doc/_module.md")]

use crate::contract::{ChildrenSurface, SlotCardinality};

#[doc = include_str!("slot_surface.doc/SlotSurface.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotSurface {
    name: &'static str,
    cardinality: SlotCardinality,
    children: ChildrenSurface,
}

impl SlotSurface {
    #[doc = include_str!("slot_surface.doc/new.md")]
    #[must_use]
    pub const fn new(name: &'static str, cardinality: SlotCardinality) -> Self {
        Self {
            name,
            cardinality,
            children: ChildrenSurface::Any,
        }
    }

    #[doc = include_str!("slot_surface.doc/allowing_child.md")]
    #[must_use]
    pub fn allowing_child(mut self, tag: &'static str) -> Self {
        match &mut self.children {
            ChildrenSurface::Only(tags) => tags.push(tag),
            _ => self.children = ChildrenSurface::Only(vec![tag]),
        }
        self
    }

    #[doc = include_str!("slot_surface.doc/name.md")]
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    #[doc = include_str!("slot_surface.doc/cardinality.md")]
    #[must_use]
    pub const fn cardinality(&self) -> SlotCardinality {
        self.cardinality
    }

    #[doc = include_str!("slot_surface.doc/children.md")]
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
