#![doc = include_str!("selection_anchor.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("selection_anchor.doc/SelectionAnchor.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SelectionAnchor {
    #[doc = include_str!("selection_anchor.doc/anchor.md")]
    anchor: NodeId,
    #[doc = include_str!("selection_anchor.doc/lead.md")]
    lead: NodeId,
}

impl SelectionAnchor {
    #[doc = include_str!("selection_anchor.doc/new.md")]
    #[must_use]
    pub const fn new(anchor: NodeId, lead: NodeId) -> Self {
        Self { anchor, lead }
    }

    #[doc = include_str!("selection_anchor.doc/collapsed.md")]
    #[must_use]
    pub const fn collapsed(node: NodeId) -> Self {
        Self {
            anchor: node,
            lead: node,
        }
    }

    #[doc = include_str!("selection_anchor.doc/anchor_2.md")]
    #[must_use]
    pub const fn anchor(&self) -> NodeId {
        self.anchor
    }

    #[doc = include_str!("selection_anchor.doc/lead_2.md")]
    #[must_use]
    pub const fn lead(&self) -> NodeId {
        self.lead
    }

    #[doc = include_str!("selection_anchor.doc/is_collapsed.md")]
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
