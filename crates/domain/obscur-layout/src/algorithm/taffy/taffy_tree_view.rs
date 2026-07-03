//! `TaffyTreeView`: the internal Taffy layout provider (spec §3.6).
//!
//! This is the ONLY type of the Taffy integration with a public API, and that API is
//! expressed **entirely in obscur types** — [`NodeId`], [`LayoutStyle`], [`LayoutBox`],
//! and [`Size`]. Taffy is spec-sanctioned here (§3.6: "Taffy integration должна
//! оставаться внутренней") purely as a pure layout-algorithm library, and it must
//! stay internal:
//!
//! * No public signature in this module (or the crate) mentions a `taffy::` type.
//! * `taffy::TaffyTree`, `taffy::NodeId`, and `taffy::Style` live only in private
//!   fields and private locals.
//! * The style/result bridges ([`TaffyStyleMapper::to_taffy`],
//!   [`TaffyResultMapper::to_layout_box`]) are `pub(crate)` for the same reason.
//!
//! Callers build a tree by inserting leaves and containers (children first), call
//! [`TaffyTreeView::compute`] with an available [`Size`], then read back each node's
//! arranged [`LayoutBox`] via [`TaffyTreeView::layout_box`].

use core::error::Error;
use core::fmt;
use std::collections::HashMap;

use super::{TaffyResultMapper, TaffyStyleMapper};
use crate::model::{LayoutBox, LayoutStyle};
use obscur_geometry::Size;
use obscur_identity::NodeId;

/// An internal Taffy-backed layout tree keyed by obscur [`NodeId`]s.
///
/// See the module documentation for the §3.6 internal-provider contract. The public
/// surface is obscur-typed end to end; taffy types never escape.
pub struct TaffyTreeView {
    /// The backing taffy tree (no per-node context).
    tree: taffy::TaffyTree<()>,
    /// Maps obscur node ids to the taffy node ids allocated for them.
    ids: HashMap<NodeId, taffy::NodeId>,
    /// Bridges obscur [`LayoutStyle`] to `taffy::Style`.
    style_mapper: TaffyStyleMapper,
    /// Bridges computed `taffy::Layout` back to obscur [`LayoutBox`].
    result_mapper: TaffyResultMapper,
}

impl TaffyTreeView {
    /// Creates an empty tree view.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tree: taffy::TaffyTree::new(),
            ids: HashMap::new(),
            style_mapper: TaffyStyleMapper::new(),
            result_mapper: TaffyResultMapper::new(),
        }
    }

    /// Inserts a childless leaf node with the given style.
    ///
    /// # Errors
    ///
    /// Returns [`TaffyLayoutError::Taffy`] if the backing tree rejects the node.
    pub fn insert_leaf(
        &mut self,
        node: NodeId,
        style: &LayoutStyle,
    ) -> Result<(), TaffyLayoutError> {
        let taffy_style = self.style_mapper.to_taffy(style);
        let taffy_id = self
            .tree
            .new_leaf(taffy_style)
            .map_err(|error| TaffyLayoutError::from_taffy(&error))?;
        self.ids.insert(node, taffy_id);
        Ok(())
    }

    /// Inserts a container node over already-inserted children.
    ///
    /// Every child in `children` must have been inserted first (children before their
    /// parent); its taffy id is resolved from the id map.
    ///
    /// # Errors
    ///
    /// * [`TaffyLayoutError::UnknownChild`] if a child id has not been inserted yet.
    /// * [`TaffyLayoutError::Taffy`] if the backing tree rejects the node.
    pub fn insert_container(
        &mut self,
        node: NodeId,
        style: &LayoutStyle,
        children: &[NodeId],
    ) -> Result<(), TaffyLayoutError> {
        let mut child_taffy_ids = Vec::with_capacity(children.len());
        for child in children {
            let taffy_id = self
                .ids
                .get(child)
                .copied()
                .ok_or(TaffyLayoutError::UnknownChild(*child))?;
            child_taffy_ids.push(taffy_id);
        }
        let taffy_style = self.style_mapper.to_taffy(style);
        let taffy_id = self
            .tree
            .new_with_children(taffy_style, &child_taffy_ids)
            .map_err(|error| TaffyLayoutError::from_taffy(&error))?;
        self.ids.insert(node, taffy_id);
        Ok(())
    }

    /// Computes the layout of the tree rooted at `root` against an available [`Size`].
    ///
    /// The available width/height are passed to taffy as definite available space.
    ///
    /// # Errors
    ///
    /// * [`TaffyLayoutError::UnknownNode`] if `root` was never inserted.
    /// * [`TaffyLayoutError::Taffy`] if taffy fails to compute the layout.
    pub fn compute(&mut self, root: NodeId, available: Size) -> Result<(), TaffyLayoutError> {
        let root_id = self
            .ids
            .get(&root)
            .copied()
            .ok_or(TaffyLayoutError::UnknownNode(root))?;
        self.tree
            .compute_layout(
                root_id,
                taffy::Size {
                    width: taffy::AvailableSpace::Definite(available.width),
                    height: taffy::AvailableSpace::Definite(available.height),
                },
            )
            .map_err(|error| TaffyLayoutError::from_taffy(&error))
    }

    /// Returns the arranged [`LayoutBox`] of `node`, if it has been inserted and its
    /// layout has been computed.
    ///
    /// Returns [`None`] for unknown nodes or when no computed layout is available.
    #[must_use]
    pub fn layout_box(&self, node: NodeId) -> Option<LayoutBox> {
        let taffy_id = self.ids.get(&node).copied()?;
        self.tree
            .layout(taffy_id)
            .ok()
            .map(|layout| self.result_mapper.to_layout_box(layout))
    }

    /// Returns `true` if `node` has been inserted into the tree.
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.ids.contains_key(&node)
    }

    /// The number of inserted nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Returns `true` if no nodes have been inserted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

impl Default for TaffyTreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for TaffyTreeView {
    /// Hand-written: `taffy::TaffyTree` is not `Debug`, so only the node count is shown.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaffyTreeView")
            .field("nodes", &self.ids.len())
            .finish_non_exhaustive()
    }
}

/// An error produced by the internal Taffy layout provider.
///
/// All variants reference obscur [`NodeId`]s; the underlying taffy error, when any, is
/// flattened into an opaque message so `taffy::TaffyError` does not leak (§3.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaffyLayoutError {
    /// The requested root/node was never inserted into the tree.
    UnknownNode(NodeId),
    /// A container referenced a child that had not been inserted yet.
    UnknownChild(NodeId),
    /// The backing taffy engine reported an error (message preserved).
    Taffy(Box<str>),
}

impl TaffyLayoutError {
    /// Flattens a `taffy::TaffyError` into an obscur-typed [`TaffyLayoutError::Taffy`].
    fn from_taffy(error: &taffy::TaffyError) -> Self {
        Self::Taffy(error.to_string().into_boxed_str())
    }
}

impl fmt::Display for TaffyLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownNode(node) => write!(f, "unknown layout node {node:?}"),
            Self::UnknownChild(node) => write!(f, "unknown child layout node {node:?}"),
            Self::Taffy(message) => write!(f, "taffy layout error: {message}"),
        }
    }
}

impl Error for TaffyLayoutError {}

#[cfg(test)]
mod tests {
    use super::{TaffyLayoutError, TaffyTreeView};
    use crate::model::{Dimension, LayoutDisplay, LayoutStyle};
    use obscur_geometry::Size;
    use obscur_identity::NodeId;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn row_flex_places_children_side_by_side() {
        let mut view = TaffyTreeView::new();

        let child_style = LayoutStyle::default()
            .with_width(Dimension::Px(100.0))
            .with_height(Dimension::Px(50.0));
        let a = node(1);
        let b = node(2);
        let root = node(3);

        view.insert_leaf(a, &child_style).unwrap();
        view.insert_leaf(b, &child_style).unwrap();

        let container_style = LayoutStyle::default()
            .with_display(LayoutDisplay::Flex)
            .with_width(Dimension::Px(300.0))
            .with_height(Dimension::Px(50.0));
        view.insert_container(root, &container_style, &[a, b]).unwrap();

        view.compute(root, Size::new(300.0, 50.0)).unwrap();

        let box_a = view.layout_box(a).unwrap();
        let box_b = view.layout_box(b).unwrap();
        let box_root = view.layout_box(root).unwrap();

        // Side by side, top-aligned: A at x=0, B at x=100, both y=0.
        assert_eq!(box_a.origin().x, 0.0);
        assert_eq!(box_a.origin().y, 0.0);
        assert_eq!(box_b.origin().x, 100.0);
        assert_eq!(box_b.origin().y, 0.0);

        assert_eq!(box_a.size(), Size::new(100.0, 50.0));
        assert_eq!(box_b.size(), Size::new(100.0, 50.0));

        // Root spans the available space.
        assert_eq!(box_root.size(), Size::new(300.0, 50.0));
    }

    #[test]
    fn compute_unknown_root_errors() {
        let mut view = TaffyTreeView::new();
        let err = view.compute(node(42), Size::new(10.0, 10.0)).unwrap_err();
        assert_eq!(err, TaffyLayoutError::UnknownNode(node(42)));
    }

    #[test]
    fn container_unknown_child_errors() {
        let mut view = TaffyTreeView::new();
        let err = view
            .insert_container(node(1), &LayoutStyle::default(), &[node(2)])
            .unwrap_err();
        assert_eq!(err, TaffyLayoutError::UnknownChild(node(2)));
    }

    #[test]
    fn contains_len_is_empty_track_inserts() {
        let mut view = TaffyTreeView::new();
        assert!(view.is_empty());
        assert_eq!(view.len(), 0);
        assert!(!view.contains(node(1)));

        view.insert_leaf(node(1), &LayoutStyle::default()).unwrap();
        assert!(!view.is_empty());
        assert_eq!(view.len(), 1);
        assert!(view.contains(node(1)));
    }

    #[test]
    fn layout_box_unknown_node_is_none() {
        let view = TaffyTreeView::new();
        assert!(view.layout_box(node(7)).is_none());
    }
}
