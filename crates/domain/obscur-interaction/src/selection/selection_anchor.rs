//! The [`SelectionAnchor`] holding the fixed anchor and moving lead of a range
//! selection (item-selection model).

use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/selection/selection_anchor.doc/SelectionAnchor.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SelectionAnchor {
    /// The fixed end of the range, set on the initial click.
    anchor: NodeId,
    /// The moving end of the range: the currently active item.
    lead: NodeId,
}

impl SelectionAnchor {
    /// Creates an anchor spanning from `anchor` to `lead`.
    #[must_use]
    pub const fn new(anchor: NodeId, lead: NodeId) -> Self {
        Self { anchor, lead }
    }

    /// Creates a collapsed anchor where both endpoints are `node`.
    #[must_use]
    pub const fn collapsed(node: NodeId) -> Self {
        Self {
            anchor: node,
            lead: node,
        }
    }

    /// Returns the fixed end of the range.
    #[must_use]
    pub const fn anchor(&self) -> NodeId {
        self.anchor
    }

    /// Returns the moving end of the range (the active item).
    #[must_use]
    pub const fn lead(&self) -> NodeId {
        self.lead
    }

    /// Returns whether the anchor and lead coincide (a single-item span).
    #[must_use]
    pub const fn is_collapsed(&self) -> bool {
        self.anchor.raw() == self.lead.raw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn new_exposes_both_ends() {
        let a = SelectionAnchor::new(node(1), node(5));
        assert_eq!(a.anchor(), node(1));
        assert_eq!(a.lead(), node(5));
        assert!(!a.is_collapsed());
    }

    #[test]
    fn collapsed_puts_both_ends_on_one_node() {
        let a = SelectionAnchor::collapsed(node(3));
        assert_eq!(a.anchor(), node(3));
        assert_eq!(a.lead(), node(3));
        assert!(a.is_collapsed());
    }
}
