#![doc = include_str!("tree_change.doc/_module.md")]

use obscur_identity::NodeId;

use crate::node::NodeKind;

use super::StagedRef;

#[doc = include_str!("tree_change.doc/TreeChange.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeChange {
    #[doc = include_str!("tree_change.doc/CreateNode.md")]
    CreateNode {
        #[doc = include_str!("tree_change.doc/kind.md")]
        kind: NodeKind,
        #[doc = include_str!("tree_change.doc/parent.md")]
        parent: StagedRef,
        #[doc = include_str!("tree_change.doc/index.md")]
        index: usize,
    },
    #[doc = include_str!("tree_change.doc/RemoveSubtree.md")]
    RemoveSubtree {
        #[doc = include_str!("tree_change.doc/target.md")]
        target: NodeId,
    },
    #[doc = include_str!("tree_change.doc/MoveChild.md")]
    MoveChild {
        #[doc = include_str!("tree_change.doc/target_2.md")]
        target: NodeId,
        #[doc = include_str!("tree_change.doc/index_2.md")]
        index: usize,
    },
    #[doc = include_str!("tree_change.doc/SetText.md")]
    SetText {
        #[doc = include_str!("tree_change.doc/target_3.md")]
        target: NodeId,
        #[doc = include_str!("tree_change.doc/content.md")]
        content: Box<str>,
    },
    #[doc = include_str!("tree_change.doc/SetRoot.md")]
    SetRoot {
        #[doc = include_str!("tree_change.doc/target_4.md")]
        target: StagedRef,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changes_are_equatable() {
        let a = TreeChange::RemoveSubtree {
            target: NodeId::from_raw(1),
        };
        let b = TreeChange::RemoveSubtree {
            target: NodeId::from_raw(1),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn create_carries_kind_and_ref() {
        let c = TreeChange::CreateNode {
            kind: NodeKind::Root,
            parent: StagedRef::New(0),
            index: 2,
        };
        let extracted = match c {
            TreeChange::CreateNode { parent, index, .. } => Some((parent, index)),
            _ => None,
        };
        assert_eq!(extracted, Some((StagedRef::New(0), 2)));
    }
}
