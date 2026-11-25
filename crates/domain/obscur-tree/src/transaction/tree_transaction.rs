//! `TreeTransaction`: a validated, atomic batch of staged tree mutations.

use std::collections::HashMap;

use obscur_identity::NodeId;

use crate::node::{LogicalParent, NodeKind, VisualParent};
use crate::tree::{RetainedTree, Subtree};

use super::{StagedRef, TreeChange, TreeChangeSet, TreeCommit, TxError};

#[doc = include_str!("../../docs/api/transaction/tree_transaction.doc/TreeTransaction.md")]
#[derive(Debug, Clone, Default)]
pub struct TreeTransaction {
    /// The ordered batch of staged changes.
    change_set: TreeChangeSet,
}

impl TreeTransaction {
    /// Creates an empty transaction with no staged changes.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stages creation of a node of `kind` under `parent` at child position
    /// `index`, and returns a [`StagedRef::New`] naming it so later changes can
    /// reference the not-yet-created node.
    pub fn create_node(&mut self, kind: NodeKind, parent: StagedRef, index: usize) -> StagedRef {
        let new_index = self.change_set.creation_count();
        self.change_set.push(TreeChange::CreateNode {
            kind,
            parent,
            index,
        });
        StagedRef::New(new_index)
    }

    /// Stages removal of `target` and its whole subtree.
    pub fn remove_subtree(&mut self, target: NodeId) {
        self.change_set.push(TreeChange::RemoveSubtree { target });
    }

    /// Stages repositioning of the existing child `target` to `index` within
    /// its current parent.
    pub fn move_child(&mut self, target: NodeId, index: usize) {
        self.change_set
            .push(TreeChange::MoveChild { target, index });
    }

    /// Stages replacement of the text content of the existing text node
    /// `target`.
    pub fn set_text(&mut self, target: NodeId, content: impl Into<Box<str>>) {
        self.change_set.push(TreeChange::SetText {
            target,
            content: content.into(),
        });
    }

    /// Stages designation of `target` as the tree root.
    pub fn set_root(&mut self, target: StagedRef) {
        self.change_set.push(TreeChange::SetRoot { target });
    }

    /// Returns the staged batch for inspection.
    #[must_use]
    pub fn changes(&self) -> &TreeChangeSet {
        &self.change_set
    }

    /// Returns the number of staged changes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.change_set.len()
    }

    /// Returns `true` if nothing has been staged.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.change_set.is_empty()
    }

    #[doc = include_str!("../../docs/api/transaction/tree_transaction.doc/cancel.md")]
    pub fn cancel(self) {
        // The staged `TreeChangeSet` is dropped here. Nothing else to do: the
        // tree was never borrowed, so there is nothing to roll back.
    }

    #[doc = include_str!("../../docs/api/transaction/tree_transaction.doc/validate.md")]
    pub fn validate(&self, tree: &RetainedTree) -> Result<(), TxError> {
        let mut creation_count: u32 = 0;
        let mut projected_child_count: HashMap<StagedRef, usize> = HashMap::new();

        for change in self.change_set.changes() {
            match change {
                TreeChange::CreateNode { parent, index, .. } => {
                    Self::require_defined(*parent, tree, creation_count)?;
                    let count = Self::projected_count(&mut projected_child_count, *parent, tree);
                    if *index > count {
                        return Err(TxError::IndexOutOfRange {
                            index: *index,
                            child_count: count,
                        });
                    }
                    projected_child_count.insert(*parent, count + 1);
                    projected_child_count.insert(StagedRef::New(creation_count), 0);
                    creation_count = creation_count.saturating_add(1);
                }
                TreeChange::RemoveSubtree { target } => {
                    if !tree.contains(*target) {
                        return Err(TxError::StaleNode(*target));
                    }
                }
                TreeChange::MoveChild { target, index } => {
                    if !tree.contains(*target) {
                        return Err(TxError::StaleNode(*target));
                    }
                    if let Some(count) = Self::live_parent_child_count(tree, *target)
                        && *index >= count
                    {
                        return Err(TxError::IndexOutOfRange {
                            index: *index,
                            child_count: count,
                        });
                    }
                }
                TreeChange::SetText { target, .. } => match tree.get(*target) {
                    None => return Err(TxError::StaleNode(*target)),
                    Some(node) if !node.kind().is_text() => {
                        return Err(TxError::NotTextNode(*target));
                    }
                    Some(_) => {}
                },
                TreeChange::SetRoot { target } => {
                    Self::require_defined(*target, tree, creation_count)?;
                }
            }
        }
        Ok(())
    }

    #[doc = include_str!("../../docs/api/transaction/tree_transaction.doc/commit.md")]
    pub fn commit(self, tree: &mut RetainedTree) -> Result<TreeCommit, TxError> {
        self.validate(tree)?;

        let base_version = tree.version();
        let changes = self.change_set.into_changes();

        // `created[i]` is the concrete id for `StagedRef::New(i)`.
        let mut created: Vec<NodeId> = Vec::new();
        let mut removed: Vec<NodeId> = Vec::new();
        let mut moved: Vec<NodeId> = Vec::new();
        let mut updated: Vec<NodeId> = Vec::new();
        let mut new_root: Option<NodeId> = None;

        for change in changes {
            match change {
                TreeChange::CreateNode {
                    kind,
                    parent,
                    index,
                } => {
                    let id = tree.alloc(kind);
                    if let Some(parent_id) = Self::resolve(parent, &created) {
                        if let Some(node) = tree.get_mut(id) {
                            node.set_logical_parent(LogicalParent::of(parent_id));
                            node.set_visual_parent(VisualParent::of(parent_id));
                        }
                        if let Some(parent_node) = tree.get_mut(parent_id) {
                            let at = index.min(parent_node.children.len());
                            parent_node.children.insert(at, id);
                        }
                        tree.sync_element_children(parent_id);
                    }
                    created.push(id);
                }
                TreeChange::RemoveSubtree { target } => {
                    // Detach from the current parent so no dangling link remains.
                    if let Some(node) = tree.get(target)
                        && let Some(parent_id) = node.logical_parent().parent()
                    {
                        if let Some(parent_node) = tree.get_mut(parent_id) {
                            parent_node.remove_child(target);
                        }
                        tree.sync_element_children(parent_id);
                    }
                    // Remove leaves first (reverse of pre-order).
                    let ids = Subtree::new(target).descendants(tree);
                    for id in ids.into_iter().rev() {
                        if tree.remove(id).is_some() {
                            removed.push(id);
                        }
                    }
                }
                TreeChange::MoveChild { target, index } => {
                    if let Some(parent_id) =
                        tree.get(target).and_then(|n| n.logical_parent().parent())
                        && let Some(parent_node) = tree.get_mut(parent_id)
                        && parent_node.remove_child(target)
                    {
                        let at = index.min(parent_node.children.len());
                        parent_node.children.insert(at, target);
                        moved.push(target);
                        tree.sync_element_children(parent_id);
                    }
                }
                TreeChange::SetText { target, content } => {
                    let parent_id = tree.get(target).and_then(|n| n.logical_parent().parent());
                    if let Some(node) = tree.get_mut(target) {
                        node.kind = NodeKind::Text { content };
                        updated.push(target);
                    }
                    // A node that becomes text is no longer a component-tree child
                    // of its parent; refresh the parent's element-only projection.
                    if let Some(parent_id) = parent_id {
                        tree.sync_element_children(parent_id);
                    }
                }
                TreeChange::SetRoot { target } => {
                    if let Some(id) = Self::resolve(target, &created) {
                        tree.set_root(id);
                        new_root = Some(id);
                    }
                }
            }
        }

        let committed_version = tree.version();
        Ok(TreeCommit::new(
            base_version,
            committed_version,
            created,
            removed,
            moved,
            updated,
            new_root,
        ))
    }

    /// Ensures a [`StagedRef`] is defined at this point in the batch: an
    /// existing id must be live, a new ref must name an earlier creation.
    fn require_defined(
        r: StagedRef,
        tree: &RetainedTree,
        creation_count: u32,
    ) -> Result<(), TxError> {
        match r {
            StagedRef::Existing(id) => {
                if tree.contains(id) {
                    Ok(())
                } else {
                    Err(TxError::StaleNode(id))
                }
            }
            StagedRef::New(i) => {
                if i < creation_count {
                    Ok(())
                } else {
                    Err(TxError::UndefinedNewRef(i))
                }
            }
        }
    }

    /// Returns the projected child count for `r`, initializing it lazily from
    /// the live tree (existing) or to `0` (new).
    fn projected_count(
        projected: &mut HashMap<StagedRef, usize>,
        r: StagedRef,
        tree: &RetainedTree,
    ) -> usize {
        if let Some(count) = projected.get(&r) {
            return *count;
        }
        let base = match r {
            StagedRef::Existing(id) => tree.get(id).map_or(0, |n| n.child_nodes().len()),
            StagedRef::New(_) => 0,
        };
        projected.insert(r, base);
        base
    }

    /// Returns the live child count of `target`'s logical parent, if that
    /// parent is live; `None` when `target` has no live parent.
    fn live_parent_child_count(tree: &RetainedTree, target: NodeId) -> Option<usize> {
        let parent_id = tree.get(target)?.logical_parent().parent()?;
        Some(tree.get(parent_id)?.child_nodes().len())
    }

    /// Resolves a [`StagedRef`] to a concrete id at apply time.
    fn resolve(r: StagedRef, created: &[NodeId]) -> Option<NodeId> {
        match r {
            StagedRef::Existing(id) => Some(id),
            StagedRef::New(i) => created.get(i as usize).copied(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;

    fn element(raw: u64) -> NodeKind {
        NodeKind::Element {
            component: ComponentId::from_raw(raw),
        }
    }

    /// A tree with a single root node, returning `(tree, root_id)`.
    fn tree_with_root() -> (RetainedTree, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        (tree, root)
    }

    #[test]
    fn staging_never_touches_the_tree() {
        let (tree, root) = tree_with_root();
        let version_before = tree.version();
        let len_before = tree.len();

        let mut tx = TreeTransaction::new();
        let child = tx.create_node(element(1), StagedRef::existing(root), 0);
        tx.create_node(element(2), child, 0);
        tx.remove_subtree(root);
        tx.set_root(child);

        // The tree is untouched by staging alone.
        assert_eq!(tree.version(), version_before);
        assert_eq!(tree.len(), len_before);
        assert_eq!(tx.len(), 4);
        assert!(!tx.is_empty());
    }

    #[test]
    fn cancel_guarantees_no_mutation() {
        let (tree, root) = tree_with_root();
        let version_before = tree.version();
        let len_before = tree.len();
        let root_before = tree.root();

        let mut tx = TreeTransaction::new();
        tx.create_node(element(1), StagedRef::existing(root), 0);
        tx.remove_subtree(root);

        // Provisional cancel: consumes the transaction, never borrows the tree.
        tx.cancel();

        assert_eq!(tree.version(), version_before);
        assert_eq!(tree.len(), len_before);
        assert_eq!(tree.root(), root_before);
    }

    #[test]
    fn failed_commit_leaves_tree_untouched() {
        let (mut tree, root) = tree_with_root();
        // Add one live child so there is real structure to disturb.
        let existing_child = tree.alloc(element(9));
        if let Some(node) = tree.get_mut(root) {
            node.push_child(existing_child);
        }
        if let Some(node) = tree.get_mut(existing_child) {
            node.set_logical_parent(LogicalParent::of(root));
        }

        let version_before = tree.version();
        let len_before = tree.len();
        let root_children_before: Vec<NodeId> = tree
            .get(root)
            .map(|n| n.child_nodes().to_vec())
            .unwrap_or_default();

        let mut tx = TreeTransaction::new();
        // A perfectly valid create staged first...
        tx.create_node(element(1), StagedRef::existing(root), 0);
        // ...followed by an invalid op: set-text on a non-text node.
        tx.set_text(existing_child, "nope");

        let result = tx.commit(&mut tree);
        assert_eq!(result, Err(TxError::NotTextNode(existing_child)));

        // Atomicity: the earlier valid create was NOT applied because validation
        // covers the whole batch before anything is applied.
        assert_eq!(tree.version(), version_before);
        assert_eq!(tree.len(), len_before);
        assert_eq!(
            tree.get(root).map(|n| n.child_nodes().to_vec()),
            Some(root_children_before)
        );
    }

    #[test]
    fn commit_applies_creates_and_resolves_new_refs() {
        let (mut tree, root) = tree_with_root();

        let mut tx = TreeTransaction::new();
        let parent = tx.create_node(element(1), StagedRef::existing(root), 0);
        let _child_a = tx.create_node(element(2), parent, 0);
        let _child_b = tx.create_node(element(3), parent, 1);
        tx.set_root(parent);

        let commit = tx.commit(&mut tree).expect("valid batch commits");

        assert_eq!(commit.created().len(), 3);
        let parent_id = commit.resolve_new(0).expect("parent created");
        let child_a_id = commit.resolve_new(1).expect("child a created");
        let child_b_id = commit.resolve_new(2).expect("child b created");

        // Parent is under root; its children are [a, b] in staged order.
        assert_eq!(
            tree.get(root).map(|n| n.child_nodes().to_vec()),
            Some(vec![parent_id])
        );
        assert_eq!(
            tree.get(parent_id).map(|n| n.child_nodes().to_vec()),
            Some(vec![child_a_id, child_b_id])
        );
        // Parents were wired on the created nodes.
        assert_eq!(
            tree.get(child_a_id)
                .map(crate::node::RetainedNode::logical_parent),
            Some(LogicalParent::of(parent_id))
        );
        assert_eq!(commit.new_root(), Some(parent_id));
        assert_eq!(tree.root(), Some(parent_id));
        assert!(commit.committed_version() != commit.base_version());
    }

    #[test]
    fn commit_removes_subtree() {
        let (mut tree, root) = tree_with_root();
        let mut tx = TreeTransaction::new();
        let a = tx.create_node(element(1), StagedRef::existing(root), 0);
        tx.create_node(element(2), a, 0);
        let commit = tx.commit(&mut tree).expect("commit");
        let a_id = commit.resolve_new(0).expect("a");

        let mut tx2 = TreeTransaction::new();
        tx2.remove_subtree(a_id);
        let commit2 = tx2.commit(&mut tree).expect("remove commit");

        assert_eq!(commit2.removed().len(), 2);
        assert!(!tree.contains(a_id));
        // Root no longer links the removed child.
        assert_eq!(
            tree.get(root).map(|n| n.child_nodes().to_vec()),
            Some(vec![])
        );
    }

    #[test]
    fn commit_moves_and_sets_text() {
        let (mut tree, root) = tree_with_root();
        let mut tx = TreeTransaction::new();
        let _a = tx.create_node(element(1), StagedRef::existing(root), 0);
        let _b = tx.create_node(element(2), StagedRef::existing(root), 1);
        let _t = tx.create_node(
            NodeKind::Text {
                content: "old".into(),
            },
            StagedRef::existing(root),
            2,
        );
        let commit = tx.commit(&mut tree).expect("commit");
        let a_id = commit.resolve_new(0).expect("a");
        let b_id = commit.resolve_new(1).expect("b");
        let t_id = commit.resolve_new(2).expect("t");

        // Root children start as [a, b, t]. Move a to index 1 -> [b, a, t].
        let mut tx2 = TreeTransaction::new();
        tx2.move_child(a_id, 1);
        tx2.set_text(t_id, "new");
        let commit2 = tx2.commit(&mut tree).expect("commit2");

        assert_eq!(commit2.moved(), &[a_id]);
        assert_eq!(commit2.updated(), &[t_id]);
        assert_eq!(
            tree.get(root).map(|n| n.child_nodes().to_vec()),
            Some(vec![b_id, a_id, t_id])
        );
        assert_eq!(tree.get(t_id).and_then(|n| n.kind().text()), Some("new"));
    }

    #[test]
    fn validate_rejects_stale_and_undefined_and_out_of_range() {
        let (tree, root) = tree_with_root();

        // Stale target.
        let mut tx = TreeTransaction::new();
        tx.remove_subtree(NodeId::from_raw(9_999));
        assert_eq!(
            tx.validate(&tree),
            Err(TxError::StaleNode(NodeId::from_raw(9_999)))
        );

        // Undefined new ref: SetRoot on New(0) with no creation staged.
        let mut tx = TreeTransaction::new();
        tx.set_root(StagedRef::New(0));
        assert_eq!(tx.validate(&tree), Err(TxError::UndefinedNewRef(0)));

        // Index out of range: root has 0 children, index 1 invalid on create.
        let mut tx = TreeTransaction::new();
        tx.create_node(element(1), StagedRef::existing(root), 1);
        assert_eq!(
            tx.validate(&tree),
            Err(TxError::IndexOutOfRange {
                index: 1,
                child_count: 0,
            })
        );
    }

    #[test]
    fn projected_count_allows_sequential_inserts() {
        let (tree, root) = tree_with_root();
        let mut tx = TreeTransaction::new();
        // Into a fresh parent: index 0 then 1 are both valid via projection.
        let parent = tx.create_node(element(1), StagedRef::existing(root), 0);
        tx.create_node(element(2), parent, 0);
        tx.create_node(element(3), parent, 1);
        assert_eq!(tx.validate(&tree), Ok(()));
    }
}
