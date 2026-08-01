#![doc = include_str!("semantic_relation.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("semantic_relation.doc/SemanticRelation.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticRelation {
    #[doc = include_str!("semantic_relation.doc/LabelledBy.md")]
    LabelledBy(NodeId),
    #[doc = include_str!("semantic_relation.doc/DescribedBy.md")]
    DescribedBy(NodeId),
    #[doc = include_str!("semantic_relation.doc/Controls.md")]
    Controls(NodeId),
    #[doc = include_str!("semantic_relation.doc/Owns.md")]
    Owns(NodeId),
    #[doc = include_str!("semantic_relation.doc/FlowsTo.md")]
    FlowsTo(NodeId),
    #[doc = include_str!("semantic_relation.doc/MemberOf.md")]
    MemberOf(NodeId),
    #[doc = include_str!("semantic_relation.doc/ActiveDescendant.md")]
    ActiveDescendant(NodeId),
}

impl SemanticRelation {
    #[doc = include_str!("semantic_relation.doc/target.md")]
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

    #[doc = include_str!("semantic_relation.doc/kind_name.md")]
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
