#![doc = include_str!("layer.doc/_module.md")]

use obscur_identity::NodeId;

use crate::node::NodeFlags;
use crate::tree::RetainedTree;

use super::{LayerId, StackingContext};

#[doc = include_str!("layer.doc/Layer.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Layer {
    #[doc = include_str!("layer.doc/id.md")]
    id: LayerId,
    #[doc = include_str!("layer.doc/root.md")]
    root: NodeId,
    #[doc = include_str!("layer.doc/stacking_context.md")]
    stacking_context: StackingContext,
}

impl Layer {
    #[doc = include_str!("layer.doc/new.md")]
    #[must_use]
    pub const fn new(id: LayerId, root: NodeId, z_index: i32) -> Self {
        Self {
            id,
            root,
            stacking_context: StackingContext::new(root, z_index),
        }
    }

    #[doc = include_str!("layer.doc/with_context.md")]
    #[must_use]
    pub const fn with_context(id: LayerId, stacking_context: StackingContext) -> Self {
        Self {
            id,
            root: stacking_context.root(),
            stacking_context,
        }
    }

    #[doc = include_str!("layer.doc/id_2.md")]
    #[must_use]
    pub const fn id(self) -> LayerId {
        self.id
    }

    #[doc = include_str!("layer.doc/root_2.md")]
    #[must_use]
    pub const fn root(self) -> NodeId {
        self.root
    }

    #[doc = include_str!("layer.doc/stacking_context_2.md")]
    #[must_use]
    pub const fn stacking_context(self) -> StackingContext {
        self.stacking_context
    }

    #[doc = include_str!("layer.doc/establish.md")]
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
        assert!(
            tree.get(root)
                .expect("root live")
                .has_flags(NodeFlags::LAYER_ROOT)
        );
    }

    #[test]
    fn establish_stale_root_is_a_noop() {
        let mut tree = RetainedTree::new();
        let ghost = NodeId::from_raw(u64::MAX);
        let layer = Layer::new(LayerId::from_raw(1), ghost, 0);
        assert!(!layer.establish(&mut tree));
    }
}
