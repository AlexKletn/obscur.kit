//! `MockIntrinsicMeasure`: deterministic stand-in intrinsic measurer for Phase 1.

use std::collections::HashMap;

use obscur_geometry::{BoxConstraints, Size};
use obscur_identity::NodeId;

use crate::model::IntrinsicSize;

use super::{IntrinsicMeasure, LayoutEnvironment};

/// A deterministic mock [`IntrinsicMeasure`] for the Phase-1 headless slice
/// (spec §5.13).
///
/// It lets the layout pipeline run without a real text engine or image decoder:
/// each node is assigned a fixed *natural content size* up front, and the mock
/// reports that size directly. This is a **test/dev stand-in only** and will be
/// replaced by the `obscur-text` adapter (and a replaced-content measurer) in a
/// later phase.
///
/// Nodes registered via [`set`](Self::set) return their stored natural size;
/// any unregistered node falls back to `default_size`. Because the mock has no
/// reflow, its [`intrinsic_size`](IntrinsicMeasure::intrinsic_size) reports the
/// natural size as both min-content and max-content.
///
/// The provider is deterministic and side-effect-free, satisfying the
/// [`IntrinsicMeasure`] contract: measurement reads the map but never mutates it.
#[derive(Debug, Clone, Default)]
pub struct MockIntrinsicMeasure {
    natural_sizes: HashMap<NodeId, Size>,
    default_size: Size,
}

impl MockIntrinsicMeasure {
    /// Creates an empty mock with no registered nodes and a `default_size` of
    /// [`Size::ZERO`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            natural_sizes: HashMap::new(),
            default_size: Size::ZERO,
        }
    }

    /// Creates an empty mock whose unregistered nodes report `size`.
    #[must_use]
    pub fn with_default(size: Size) -> Self {
        Self {
            natural_sizes: HashMap::new(),
            default_size: size,
        }
    }

    /// Registers `node`'s natural content size, overwriting any prior entry.
    ///
    /// Returns `&mut Self` so registrations can be chained during test setup.
    pub fn set(&mut self, node: NodeId, size: Size) -> &mut Self {
        self.natural_sizes.insert(node, size);
        self
    }

    /// The natural content size for `node`: its registered size, or
    /// `default_size` if unregistered.
    #[must_use]
    fn natural_size(&self, node: NodeId) -> Size {
        self.natural_sizes
            .get(&node)
            .copied()
            .unwrap_or(self.default_size)
    }
}

impl IntrinsicMeasure for MockIntrinsicMeasure {
    /// Returns the node's natural size (registered or default) clamped into
    /// `constraints` via [`BoxConstraints::constrain`]. The environment is
    /// unused: the mock's sizes are fixed regardless of scale or viewport.
    fn measure(&self, node: NodeId, constraints: BoxConstraints, _env: &LayoutEnvironment) -> Size {
        constraints.constrain(self.natural_size(node))
    }

    /// Returns [`IntrinsicSize::fixed`] of the node's natural size: a mock has
    /// no reflow, so min-content and max-content coincide (spec §4.1).
    fn intrinsic_size(&self, node: NodeId, _env: &LayoutEnvironment) -> IntrinsicSize {
        IntrinsicSize::fixed(self.natural_size(node))
    }
}

#[cfg(test)]
mod tests {
    use super::MockIntrinsicMeasure;
    use crate::model::IntrinsicSize;
    use crate::port::{IntrinsicMeasure, LayoutEnvironment};
    use obscur_geometry::{BoxConstraints, Size};
    use obscur_identity::NodeId;

    fn env() -> LayoutEnvironment {
        LayoutEnvironment::identity()
    }

    #[test]
    fn new_is_empty_with_zero_default() {
        let mock = MockIntrinsicMeasure::new();
        let unbounded = BoxConstraints::loose(Size::new(1000.0, 1000.0));
        assert_eq!(
            mock.measure(NodeId::from_raw(7), unbounded, &env()),
            Size::ZERO
        );
    }

    #[test]
    fn unregistered_node_uses_default() {
        let mock = MockIntrinsicMeasure::with_default(Size::new(12.0, 4.0));
        let unbounded = BoxConstraints::loose(Size::new(1000.0, 1000.0));
        assert_eq!(
            mock.measure(NodeId::from_raw(3), unbounded, &env()),
            Size::new(12.0, 4.0)
        );
    }

    #[test]
    fn registered_node_reports_its_size() {
        let mut mock = MockIntrinsicMeasure::with_default(Size::new(1.0, 1.0));
        let node = NodeId::from_raw(42);
        mock.set(node, Size::new(50.0, 20.0));
        let unbounded = BoxConstraints::loose(Size::new(1000.0, 1000.0));
        assert_eq!(mock.measure(node, unbounded, &env()), Size::new(50.0, 20.0));
    }

    #[test]
    fn measure_constrains_into_bounds() {
        let mut mock = MockIntrinsicMeasure::new();
        let node = NodeId::from_raw(1);
        mock.set(node, Size::new(200.0, 200.0));
        let bounded = BoxConstraints::loose(Size::new(80.0, 30.0));
        assert_eq!(mock.measure(node, bounded, &env()), Size::new(80.0, 30.0));
    }

    #[test]
    fn intrinsic_size_is_fixed_natural_size() {
        let mut mock = MockIntrinsicMeasure::new();
        let node = NodeId::from_raw(9);
        mock.set(node, Size::new(33.0, 11.0));
        assert_eq!(
            mock.intrinsic_size(node, &env()),
            IntrinsicSize::fixed(Size::new(33.0, 11.0))
        );
    }
}
