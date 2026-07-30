//! `SlotSurface`: the string-keyed *tooling* view of one declared slot (spec
//! §2.12).

use crate::contract::{ChildrenSurface, SlotCardinality};

/// The string-keyed *tooling* view of one named slot a component declares (spec
/// §2.12).
///
/// [`SlotContract`](crate::contract::SlotContract) is the **runtime** view: it
/// names the slot with an [`InternedSymbol`](obscur_identity::InternedSymbol)
/// and its members with [`ComponentId`](obscur_identity::ComponentId)s, neither
/// of which resolves back to a string. A `.okui` source writes `slot="header"`,
/// so the tool that checks it needs the name as written — the same split
/// [`ComponentSurface`](crate::contract::ComponentSurface) makes against the
/// typed [`ComponentContract`](crate::component::ComponentContract), and for the
/// same reason.
///
/// A slot declares the three things §2.12 names:
///
/// - `name` — the value a child's `slot="…"` must match;
/// - `cardinality` — how many children may carry that name (see
///   [`SlotCardinality`]);
/// - `children` — which child tags the slot admits (see [`ChildrenSurface`]; only
///   `Only` restricts a **slot's** membership — `Any` and `None` both leave it
///   unrestricted, because a slot that admits nothing is a slot with nothing to
///   assign to it, which its `cardinality` states better. A *component's* `None`
///   does now cross into the `.okui` compiler as a real leaf; a slot's does not).
///
/// ```
/// use obscur_view::contract::{ChildrenSurface, SlotCardinality, SlotSurface};
/// let slot = SlotSurface::new("header", SlotCardinality::ExactlyOne).allowing_child("Title");
/// assert_eq!(slot.name(), "header");
/// assert_eq!(slot.cardinality(), SlotCardinality::ExactlyOne);
/// assert_eq!(slot.children(), &ChildrenSurface::Only(vec!["Title"]));
/// ```
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
