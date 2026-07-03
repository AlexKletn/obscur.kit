//! A compositing layer that groups a subtree for composition.

use obscur_identity::NodeId;

use crate::node::NodeFlags;
use crate::tree::RetainedTree;

use super::{LayerId, StackingContext};

/// A compositing layer that groups a subtree for composition (spec §5.1).
///
/// A layer pairs an opaque [`LayerId`] with the establishing `root` node and the
/// [`StackingContext`] that fixes its paint order among siblings. This is the
/// **retained-tree-side descriptor**: it names and orders a layer, but the
/// actual compositing (surface allocation, blending, GPU work) lives in
/// `obscur-scene` / `obscur-render` later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Layer {
    /// The opaque identity of this layer.
    id: LayerId,
    /// The node that establishes this layer's stacking context.
    root: NodeId,
    /// The stacking context describing this layer's paint order.
    stacking_context: StackingContext,
}

impl Layer {
    /// Creates a layer with the given `id`, establishing `root` node, and paint
    /// order `z_index`.
    ///
    /// The [`StackingContext`] is built from `root` and `z_index`; use
    /// [`Layer::with_context`] to supply a pre-built context.
    #[must_use]
    pub const fn new(id: LayerId, root: NodeId, z_index: i32) -> Self {
        Self {
            id,
            root,
            stacking_context: StackingContext::new(root, z_index),
        }
    }

    /// Creates a layer from an explicit [`StackingContext`].
    ///
    /// The layer's `root` is taken from the context, keeping the two in sync.
    #[must_use]
    pub const fn with_context(id: LayerId, stacking_context: StackingContext) -> Self {
        Self {
            id,
            root: stacking_context.root(),
            stacking_context,
        }
    }

    /// Returns this layer's opaque identity.
    #[must_use]
    pub const fn id(self) -> LayerId {
        self.id
    }

    /// Returns the node that establishes this layer.
    #[must_use]
    pub const fn root(self) -> NodeId {
        self.root
    }

    /// Returns the stacking context describing this layer's paint order.
    #[must_use]
    pub const fn stacking_context(self) -> StackingContext {
        self.stacking_context
    }

    /// Flags the layer's root node as a stacking-context root
    /// ([`NodeFlags::LAYER_ROOT`]) on `tree`.
    ///
    /// Returns `true` if the root is live and was flagged; returns `false` (a
    /// no-op) if the root id is stale or was never valid. It is panic-free and
    /// mutates nothing when the root does not resolve.
    ///
    /// This only records the establishing flag on the retained tree; actual
    /// compositing lives in scene/render later.
    pub fn establish(self, tree: &mut RetainedTree) -> bool {
        let Some(root) = tree.get_mut(self.root) else {
            return false;
        };
        root.insert_flags(NodeFlags::LAYER_ROOT);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeKind;

    #[test]
    fn new_builds_matching_context_and_accessors() {
        let root = NodeId::from_raw(5);
        let layer = Layer::new(LayerId::from_raw(1), root, 3);
        assert_eq!(layer.id(), LayerId::from_raw(1));
        assert_eq!(layer.root(), root);
        assert_eq!(layer.stacking_context(), StackingContext::new(root, 3));
    }

    #[test]
    fn with_context_takes_root_from_context() {
        let ctx = StackingContext::new(NodeId::from_raw(8), -2);
        let layer = Layer::with_context(LayerId::from_raw(2), ctx);
        assert_eq!(layer.root(), NodeId::from_raw(8));
        assert_eq!(layer.stacking_context(), ctx);
    }

    #[test]
    fn establish_flags_live_root() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let layer = Layer::new(LayerId::from_raw(1), root, 0);
        assert!(layer.establish(&mut tree));
        assert!(tree.get(root).expect("root live").has_flags(NodeFlags::LAYER_ROOT));
    }

    #[test]
    fn establish_stale_root_is_a_noop() {
        let mut tree = RetainedTree::new();
        let ghost = NodeId::from_raw(u64::MAX);
        let layer = Layer::new(LayerId::from_raw(1), ghost, 0);
        assert!(!layer.establish(&mut tree));
    }
}
