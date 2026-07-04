//! A typed relation from one accessibility node to another (spec §7.1).

use obscur_identity::NodeId;

/// A typed relation from one accessibility node to another (spec §7.1
/// relations).
///
/// Each variant expresses an ARIA-like relationship and carries the
/// [`NodeId`] of the related retained-tree node as its single target,
/// retrievable via [`target`](Self::target). Platform adapters translate these
/// into their native relation types (e.g. `aria-labelledby`,
/// `aria-controls`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticRelation {
    /// The node is labelled by the target node.
    LabelledBy(NodeId),
    /// The node is described by the target node.
    DescribedBy(NodeId),
    /// The node controls the target node.
    Controls(NodeId),
    /// The node owns the target node as a logical child.
    Owns(NodeId),
    /// Reading flow continues to the target node.
    FlowsTo(NodeId),
    /// The node is a member of the group identified by the target.
    MemberOf(NodeId),
    /// The target is the active descendant within this node.
    ActiveDescendant(NodeId),
}

impl SemanticRelation {
    /// Returns the [`NodeId`] this relation points at.
    #[must_use]
    pub const fn target(&self) -> NodeId {
        match self {
            Self::LabelledBy(id)
            | Self::DescribedBy(id)
            | Self::Controls(id)
            | Self::Owns(id)
            | Self::FlowsTo(id)
            | Self::MemberOf(id)
            | Self::ActiveDescendant(id) => *id,
        }
    }

    /// Returns a stable, kebab-free name for the relation kind, suitable for
    /// diagnostics and adapter mapping.
    #[must_use]
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::LabelledBy(_) => "labelled_by",
            Self::DescribedBy(_) => "described_by",
            Self::Controls(_) => "controls",
            Self::Owns(_) => "owns",
            Self::FlowsTo(_) => "flows_to",
            Self::MemberOf(_) => "member_of",
            Self::ActiveDescendant(_) => "active_descendant",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_across_variants() {
        let id = NodeId::from_raw(42);
        assert_eq!(SemanticRelation::LabelledBy(id).target(), id);
        assert_eq!(SemanticRelation::DescribedBy(id).target(), id);
        assert_eq!(SemanticRelation::Controls(id).target(), id);
        assert_eq!(SemanticRelation::Owns(id).target(), id);
        assert_eq!(SemanticRelation::FlowsTo(id).target(), id);
        assert_eq!(SemanticRelation::MemberOf(id).target(), id);
        assert_eq!(SemanticRelation::ActiveDescendant(id).target(), id);
    }

    #[test]
    fn kind_names() {
        let id = NodeId::from_raw(1);
        assert_eq!(SemanticRelation::LabelledBy(id).kind_name(), "labelled_by");
        assert_eq!(
            SemanticRelation::DescribedBy(id).kind_name(),
            "described_by"
        );
        assert_eq!(SemanticRelation::Controls(id).kind_name(), "controls");
        assert_eq!(SemanticRelation::Owns(id).kind_name(), "owns");
        assert_eq!(SemanticRelation::FlowsTo(id).kind_name(), "flows_to");
        assert_eq!(SemanticRelation::MemberOf(id).kind_name(), "member_of");
        assert_eq!(
            SemanticRelation::ActiveDescendant(id).kind_name(),
            "active_descendant"
        );
    }
}
