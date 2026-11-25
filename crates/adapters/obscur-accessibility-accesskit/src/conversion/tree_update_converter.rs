//! Builds an AccessKit [`TreeUpdate`] from a [`SemanticSnapshot`] (§7.1).

use std::collections::{BTreeMap, BTreeSet};
use std::hash::{DefaultHasher, Hash, Hasher};

use accesskit::{Node, NodeId, Role, TextPosition, TextSelection, Tree, TreeUpdate};
use obscur_accessibility::semantic::{SemanticRelation, SemanticValue};
use obscur_accessibility::tree::SemanticSnapshot;
use obscur_identity::NodeId as DomainNodeId;
use unicode_segmentation::UnicodeSegmentation;

use super::NodeConverter;
use crate::error::AccessKitBridgeError;

#[doc = include_str!("../../docs/api/conversion/tree_update_converter.doc/TreeUpdateConverter.md")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TreeUpdateConverter {
    node: NodeConverter,
}

impl TreeUpdateConverter {
    /// Creates a converter over a fresh [`NodeConverter`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            node: NodeConverter::new(),
        }
    }

    /// Returns the inner [`NodeConverter`], e.g. to map ids consistently.
    #[must_use]
    pub const fn node_converter(&self) -> &NodeConverter {
        &self.node
    }

    #[doc = include_str!("../../docs/api/conversion/tree_update_converter.doc/convert.md")]
    pub fn convert(&self, snapshot: &SemanticSnapshot) -> Result<TreeUpdate, AccessKitBridgeError> {
        self.convert_with_focus(snapshot, None)
    }

    #[doc = include_str!("../../docs/api/conversion/tree_update_converter.doc/convert_with_focus.md")]
    pub fn convert_with_focus(
        &self,
        snapshot: &SemanticSnapshot,
        focus: Option<DomainNodeId>,
    ) -> Result<TreeUpdate, AccessKitBridgeError> {
        let root = snapshot.root().ok_or(AccessKitBridgeError::MissingRoot)?;

        let children = Self::reconstruct_children(snapshot, root);

        let mut used_ids: BTreeSet<_> = snapshot
            .iter()
            .map(|(id, _)| self.node.node_id(id))
            .collect();
        let mut nodes = Vec::new();
        for (id, node) in snapshot.iter().filter(|(_, node)| !node.is_ignored()) {
            let child_ids = children.get(&id).map_or(&[][..], Vec::as_slice);
            let id = self.node.node_id(id);
            let mut converted = self.node.convert(node, child_ids);
            if let (SemanticValue::Text(text), Some(selection)) =
                (node.value(), node.text_selection)
                && selection
                    .iter()
                    .all(|&offset| text.is_char_boundary(offset))
            {
                append_text_runs(
                    &mut converted,
                    id,
                    text,
                    selection,
                    &mut used_ids,
                    &mut nodes,
                );
            }
            nodes.push((id, converted));
        }

        let focus_id = focus
            .filter(|id| snapshot.get(*id).is_some_and(|n| !n.is_ignored()))
            .unwrap_or(root);

        Ok(TreeUpdate {
            nodes,
            tree: Some(Tree::new(self.node.node_id(root))),
            focus: self.node.node_id(focus_id),
        })
    }

    /// Rebuilds a parent→children map from the snapshot's own child links, adds
    /// the ownership its `Owns` relations express, then attaches every
    /// still-parentless non-ignored node to the root.
    fn reconstruct_children(
        snapshot: &SemanticSnapshot,
        root: DomainNodeId,
    ) -> BTreeMap<DomainNodeId, Vec<DomainNodeId>> {
        let mut children: BTreeMap<DomainNodeId, Vec<DomainNodeId>> = BTreeMap::new();
        let mut owned: BTreeSet<DomainNodeId> = BTreeSet::new();
        let mut visited: BTreeSet<DomainNodeId> = BTreeSet::new();

        // The captured tree's structure, walked from the root.
        visited.insert(root);
        Self::graft_links(
            snapshot,
            root,
            root,
            &mut children,
            &mut owned,
            &mut visited,
        );

        // Explicit ownership, in deterministic id order, over whatever the links
        // did not already parent.
        for (id, node) in snapshot
            .iter()
            .filter(|(id, n)| !n.is_ignored() && *id != root)
        {
            for relation in node.relations() {
                let SemanticRelation::Owns(target) = relation else {
                    continue;
                };
                let present = snapshot.get(*target).is_some_and(|n| !n.is_ignored());
                if *target != root && *target != id && present && owned.insert(*target) {
                    children.entry(id).or_default().push(*target);
                }
            }
        }

        // Any node neither linked nor owned hangs off the root.
        let orphans: Vec<DomainNodeId> = snapshot
            .iter()
            .filter(|(id, n)| !n.is_ignored() && *id != root && !owned.contains(id))
            .map(|(id, _)| id)
            .collect();
        if !orphans.is_empty() {
            children.entry(root).or_default().extend(orphans);
        }

        children
    }

    #[doc = include_str!("../../docs/api/conversion/tree_update_converter.doc/graft_links.md")]
    fn graft_links(
        snapshot: &SemanticSnapshot,
        ancestor: DomainNodeId,
        node: DomainNodeId,
        children: &mut BTreeMap<DomainNodeId, Vec<DomainNodeId>>,
        owned: &mut BTreeSet<DomainNodeId>,
        visited: &mut BTreeSet<DomainNodeId>,
    ) {
        for &child in snapshot.children(node) {
            if !visited.insert(child) {
                continue;
            }
            let Some(semantic) = snapshot.get(child) else {
                continue;
            };
            if semantic.is_ignored() {
                Self::graft_links(snapshot, ancestor, child, children, owned, visited);
            } else if owned.insert(child) {
                children.entry(ancestor).or_default().push(child);
                Self::graft_links(snapshot, child, child, children, owned, visited);
            }
        }
    }
}

fn append_text_runs(
    field: &mut Node,
    field_id: NodeId,
    text: &str,
    selection: [usize; 2],
    used_ids: &mut BTreeSet<NodeId>,
    nodes: &mut Vec<(NodeId, Node)>,
) {
    let mut children = field.children().to_vec();
    let mut positions = [None; 2];
    let mut start = 0;
    // Split on hard line breaks and keep the final empty run at a trailing break.
    for (index, line) in text
        .split_inclusive('\n')
        .chain((text.is_empty() || text.ends_with('\n')).then_some(""))
        .enumerate()
    {
        let mut hash = DefaultHasher::new();
        (field_id.0, index).hash(&mut hash);
        let mut id = NodeId(hash.finish());
        while !used_ids.insert(id) {
            id.0 = id.0.wrapping_sub(1);
        }
        let mut lengths = Vec::new();
        for grapheme in line.graphemes(true) {
            if let Ok(length) = u8::try_from(grapheme.len()) {
                lengths.push(length);
            } else {
                lengths.extend(grapheme.chars().map(|ch| ch.len_utf8() as u8));
            }
        }
        let mut end = 0;
        let ends: Vec<_> = lengths
            .iter()
            .map(|&length| {
                end += usize::from(length);
                end
            })
            .collect();
        let line_end = start + line.len();
        for (position, offset) in positions.iter_mut().zip(selection) {
            if offset >= start && offset <= line_end {
                *position = Some(TextPosition {
                    node: id,
                    character_index: ends.partition_point(|&end| end <= offset - start),
                });
            }
        }

        let mut run = Node::new(Role::TextRun);
        run.set_value(line);
        let mut word_lengths = Vec::new();
        let mut previous = 0;
        for (offset, word) in line.split_word_bound_indices() {
            let end = ends.partition_point(|&end| end <= offset + word.len());
            let mut length = end - previous;
            while length > usize::from(u8::MAX) {
                word_lengths.push(u8::MAX);
                length -= usize::from(u8::MAX);
            }
            if length > 0 {
                word_lengths.push(length as u8);
            }
            previous = end;
        }
        run.set_word_lengths(word_lengths);
        run.set_character_lengths(lengths);
        children.push(id);
        nodes.push((id, run));
        start = line_end;
    }
    field.set_children(children);
    if let [Some(anchor), Some(focus)] = positions {
        field.set_text_selection(TextSelection { anchor, focus });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_accessibility::semantic::{SemanticNode, SemanticRelation, SemanticRole};
    use obscur_accessibility::tree::SemanticTree;

    fn node(n: u64, role: SemanticRole) -> SemanticNode {
        SemanticNode::new(DomainNodeId::from_raw(n), role)
    }

    #[test]
    fn editable_selection_references_grapheme_positions_across_lines() {
        let root = DomainNodeId::from_raw(1);
        let field = DomainNodeId::from_raw(2);
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        tree.insert(node(2, SemanticRole::TextInput), Some(root));
        tree.set_root(root);
        let text = "ae\u{301}\né";
        tree.project_editing(field, text, [text.len(), 1]);
        let snapshot = SemanticSnapshot::from_tree(&tree);
        let converter = TreeUpdateConverter::new();
        let output = converter
            .convert_with_focus(&snapshot, Some(field))
            .unwrap();
        let find = |id| {
            output
                .nodes
                .iter()
                .find(|(key, _)| *key == id)
                .map(|(_, node)| node)
                .unwrap()
        };
        let input = find(output.focus);
        let selection = input.text_selection().unwrap();
        assert_eq!(selection.anchor.node, input.children()[1]);
        assert_eq!(selection.anchor.character_index, 1);
        assert_eq!(selection.focus.node, input.children()[0]);
        assert_eq!(selection.focus.character_index, 1);
        assert_eq!(find(selection.focus.node).character_lengths(), &[1, 3, 1]);
        assert_eq!(find(selection.anchor.node).character_lengths(), &[2]);
        let again = converter
            .convert_with_focus(&snapshot, Some(field))
            .unwrap();
        assert_eq!(output.nodes, again.nodes);
    }

    fn two_node_snapshot() -> SemanticSnapshot {
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        tree.insert(
            node(2, SemanticRole::Button),
            Some(DomainNodeId::from_raw(1)),
        );
        tree.set_root(DomainNodeId::from_raw(1));
        SemanticSnapshot::from_tree(&tree)
    }

    #[test]
    fn two_node_snapshot_yields_two_nodes_root_and_focus() {
        let update = TreeUpdateConverter::new()
            .convert(&two_node_snapshot())
            .expect("has root");
        assert_eq!(update.nodes.len(), 2);
        assert_eq!(update.tree.as_ref().map(|t| t.root), Some(NodeId(1)));
        // No explicit focus → focus defaults to the root.
        assert_eq!(update.focus, NodeId(1));
    }

    /// The children AccessKit was told `id` has.
    fn child_of(update: &TreeUpdate, id: NodeId) -> Vec<NodeId> {
        update
            .nodes
            .iter()
            .find(|(nid, _)| *nid == id)
            .map(|(_, n)| n.children().to_vec())
            .unwrap_or_default()
    }

    #[test]
    fn root_gets_orphan_as_child() {
        let update = TreeUpdateConverter::new()
            .convert(&two_node_snapshot())
            .expect("has root");
        assert_eq!(child_of(&update, NodeId(1)), vec![NodeId(2)]);
    }

    /// The structure a screen reader is handed must be the tree's own, not one
    /// flat level under the root: the snapshot's links are the only source a
    /// projection of the retained tree carries (it stamps no `Owns` relations),
    /// so ignoring them collapses every nesting level.
    #[test]
    fn nesting_follows_the_snapshots_child_links() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        tree.insert(node(2, SemanticRole::List), Some(DomainNodeId::from_raw(1)));
        tree.insert(
            node(3, SemanticRole::ListItem),
            Some(DomainNodeId::from_raw(2)),
        );
        tree.insert(node(4, SemanticRole::Text), Some(DomainNodeId::from_raw(3)));
        tree.set_root(DomainNodeId::from_raw(1));

        let update = TreeUpdateConverter::new()
            .convert(&SemanticSnapshot::from_tree(&tree))
            .expect("has root");

        assert_eq!(child_of(&update, NodeId(1)), vec![NodeId(2)]);
        assert_eq!(child_of(&update, NodeId(2)), vec![NodeId(3)]);
        assert_eq!(
            child_of(&update, NodeId(3)),
            vec![NodeId(4)],
            "every level of the tree survives the hand-off"
        );
    }

    /// Sibling order is authored order, so an assistive technology reads the UI
    /// in the order it was written.
    #[test]
    fn sibling_order_is_preserved() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        for id in [4, 2, 3] {
            tree.insert(
                node(id, SemanticRole::Button),
                Some(DomainNodeId::from_raw(1)),
            );
        }
        tree.set_root(DomainNodeId::from_raw(1));

        let update = TreeUpdateConverter::new()
            .convert(&SemanticSnapshot::from_tree(&tree))
            .expect("has root");
        assert_eq!(
            child_of(&update, NodeId(1)),
            vec![NodeId(4), NodeId(2), NodeId(3)],
            "insertion order, not id order"
        );
    }

    /// An ignored node publishes no AccessKit node, so it cannot parent anything
    /// — but its content is not decorative just because its wrapper is.
    #[test]
    fn an_ignored_node_is_spliced_out_and_its_children_graft_upward() {
        // root → panel → decorative wrapper → button: the button must land on the
        // panel, which is the nearest ancestor that publishes a node at all.
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        tree.insert(
            node(2, SemanticRole::Region),
            Some(DomainNodeId::from_raw(1)),
        );
        tree.insert(
            node(3, SemanticRole::Group).decorative(true),
            Some(DomainNodeId::from_raw(2)),
        );
        tree.insert(
            node(4, SemanticRole::Button),
            Some(DomainNodeId::from_raw(3)),
        );
        tree.set_root(DomainNodeId::from_raw(1));

        let update = TreeUpdateConverter::new()
            .convert(&SemanticSnapshot::from_tree(&tree))
            .expect("has root");
        assert_eq!(
            update.nodes.len(),
            3,
            "the decorative wrapper is not published"
        );
        assert_eq!(
            child_of(&update, NodeId(2)),
            vec![NodeId(4)],
            "the button keeps its place under the panel rather than vanishing \
             with its wrapper or being flung out to the root"
        );
        assert_eq!(child_of(&update, NodeId(1)), vec![NodeId(2)]);
    }

    /// A cyclic snapshot is malformed, but it must not hang the host that
    /// converts it.
    #[test]
    fn a_cyclic_link_terminates() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        tree.insert(
            node(2, SemanticRole::Group),
            Some(DomainNodeId::from_raw(1)),
        );
        // Node 2 lists node 1 (the root) as its own child.
        tree.insert(
            node(1, SemanticRole::Group),
            Some(DomainNodeId::from_raw(2)),
        );
        tree.set_root(DomainNodeId::from_raw(1));

        let update = TreeUpdateConverter::new()
            .convert(&SemanticSnapshot::from_tree(&tree))
            .expect("has root");
        assert_eq!(child_of(&update, NodeId(1)), vec![NodeId(2)]);
        assert!(child_of(&update, NodeId(2)).is_empty());
    }

    #[test]
    fn rootless_snapshot_is_missing_root() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        let snapshot = SemanticSnapshot::from_tree(&tree);
        assert_eq!(
            TreeUpdateConverter::new().convert(&snapshot),
            Err(AccessKitBridgeError::MissingRoot)
        );
    }

    #[test]
    fn ignored_nodes_are_skipped() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        tree.insert(
            node(2, SemanticRole::Button).decorative(true),
            Some(DomainNodeId::from_raw(1)),
        );
        tree.set_root(DomainNodeId::from_raw(1));
        let update = TreeUpdateConverter::new()
            .convert(&SemanticSnapshot::from_tree(&tree))
            .expect("has root");
        assert_eq!(update.nodes.len(), 1);
    }

    #[test]
    fn explicit_focus_is_honored() {
        let update = TreeUpdateConverter::new()
            .convert_with_focus(&two_node_snapshot(), Some(DomainNodeId::from_raw(2)))
            .expect("has root");
        assert_eq!(update.focus, NodeId(2));
    }

    #[test]
    fn owns_relation_builds_nested_structure() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        tree.insert(node(3, SemanticRole::ListItem), None);
        tree.insert(
            node(2, SemanticRole::List)
                .with_relation(SemanticRelation::Owns(DomainNodeId::from_raw(3))),
            None,
        );
        tree.set_root(DomainNodeId::from_raw(1));
        let update = TreeUpdateConverter::new()
            .convert(&SemanticSnapshot::from_tree(&tree))
            .expect("has root");

        // Node 3 is owned by node 2, not attached to the root.
        assert_eq!(child_of(&update, NodeId(2)), vec![NodeId(3)]);
        assert!(!child_of(&update, NodeId(1)).contains(&NodeId(3)));
    }
}
