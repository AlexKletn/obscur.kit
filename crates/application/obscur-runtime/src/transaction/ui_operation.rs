#![doc = include_str!("ui_operation.doc/_module.md")]

use obscur_identity::NodeId;

use crate::invalidation::{Invalidation, InvalidationReason};

#[doc = include_str!("ui_operation.doc/UiOperation.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiOperation {
    #[doc = include_str!("ui_operation.doc/ReplaceRoot.md")]
    ReplaceRoot,
    #[doc = include_str!("ui_operation.doc/Rerender.md")]
    Rerender {
        #[doc = include_str!("ui_operation.doc/node.md")]
        node: NodeId,
    },
    #[doc = include_str!("ui_operation.doc/InvalidateStyle.md")]
    InvalidateStyle {
        #[doc = include_str!("ui_operation.doc/node_2.md")]
        node: NodeId,
    },
    #[doc = include_str!("ui_operation.doc/InvalidateLayout.md")]
    InvalidateLayout {
        #[doc = include_str!("ui_operation.doc/node_3.md")]
        node: NodeId,
    },
}

impl UiOperation {
    #[doc = include_str!("ui_operation.doc/invalidation.md")]
    #[must_use]
    pub fn invalidation(self) -> Invalidation {
        match self {
            Self::ReplaceRoot => Invalidation::global(InvalidationReason::TreeChanged),
            Self::Rerender { node } => {
                Invalidation::new(Some(node), InvalidationReason::TreeChanged)
            }
            Self::InvalidateStyle { node } => {
                Invalidation::new(Some(node), InvalidationReason::StyleChanged)
            }
            Self::InvalidateLayout { node } => {
                Invalidation::new(Some(node), InvalidationReason::LayoutChanged)
            }
        }
    }

    #[doc = include_str!("ui_operation.doc/target.md")]
    #[must_use]
    pub const fn target(&self) -> Option<NodeId> {
        match self {
            Self::ReplaceRoot => None,
            Self::Rerender { node }
            | Self::InvalidateStyle { node }
            | Self::InvalidateLayout { node } => Some(*node),
        }
    }

    #[doc = include_str!("ui_operation.doc/is_root_replacement.md")]
    #[must_use]
    pub const fn is_root_replacement(&self) -> bool {
        matches!(self, Self::ReplaceRoot)
    }

    #[doc = include_str!("ui_operation.doc/is_rerender.md")]
    #[must_use]
    pub const fn is_rerender(&self) -> bool {
        matches!(self, Self::Rerender { .. })
    }

    #[doc = include_str!("ui_operation.doc/is_style_invalidation.md")]
    #[must_use]
    pub const fn is_style_invalidation(&self) -> bool {
        matches!(self, Self::InvalidateStyle { .. })
    }

    #[doc = include_str!("ui_operation.doc/is_layout_invalidation.md")]
    #[must_use]
    pub const fn is_layout_invalidation(&self) -> bool {
        matches!(self, Self::InvalidateLayout { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_replacement_has_no_target() {
        let op = UiOperation::ReplaceRoot;
        assert_eq!(op.target(), None);
        assert!(op.is_root_replacement());
        assert!(!op.is_rerender());
    }

    #[test]
    fn node_operations_report_their_target() {
        let node = NodeId::from_raw(7);
        for op in [
            UiOperation::Rerender { node },
            UiOperation::InvalidateStyle { node },
            UiOperation::InvalidateLayout { node },
        ] {
            assert_eq!(op.target(), Some(node));
            assert!(!op.is_root_replacement());
        }
    }

    #[test]
    fn predicates_are_mutually_exclusive() {
        let node = NodeId::from_raw(1);
        assert!(UiOperation::Rerender { node }.is_rerender());
        assert!(UiOperation::InvalidateStyle { node }.is_style_invalidation());
        assert!(UiOperation::InvalidateLayout { node }.is_layout_invalidation());
        assert!(!UiOperation::Rerender { node }.is_style_invalidation());
    }
}
