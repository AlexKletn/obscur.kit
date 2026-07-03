//! `TreeChange`: one staged tree-mutation operation.

use obscur_identity::NodeId;

use crate::node::NodeKind;

use super::StagedRef;

/// A single staged mutation of the retained tree.
///
/// A [`TreeTransaction`](super::TreeTransaction) accumulates an ordered list of
/// these. They are pure data: staging a `TreeChange` never touches the live
/// tree. The batch is validated and then applied, in order, only at
/// [`commit`](super::TreeTransaction::commit).
///
/// Nodes are referenced through a [`StagedRef`] where a change may target a node
/// created earlier *in the same transaction*, and through a live [`NodeId`]
/// where only an already-existing node can be meant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeChange {
    /// Create a node of `kind` as a child of `parent` at child position
    /// `index`. The created node is assigned the next
    /// [`StagedRef::New`] index and may be referenced by later changes.
    CreateNode {
        /// The classification of the node to create.
        kind: NodeKind,
        /// The parent under which the new node is inserted.
        parent: StagedRef,
        /// The child position within `parent` at which to insert (clamped on
        /// apply).
        index: usize,
    },
    /// Remove an existing node and its whole subtree from the tree.
    RemoveSubtree {
        /// The root of the subtree to remove; must be a live node.
        target: NodeId,
    },
    /// Reposition an existing child to `index` within its current parent's
    /// child list.
    MoveChild {
        /// The child to reposition; must be a live node.
        target: NodeId,
        /// The destination child position within the current parent (clamped
        /// on apply).
        index: usize,
    },
    /// Replace an existing text node's content.
    SetText {
        /// The text node to update; must be a live [`NodeKind::Text`] node.
        target: NodeId,
        /// The replacement text content.
        content: Box<str>,
    },
    /// Designate the tree root.
    SetRoot {
        /// The node to make the root; may be a node created earlier in this
        /// transaction.
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
