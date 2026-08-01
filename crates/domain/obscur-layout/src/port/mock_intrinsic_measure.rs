#![doc = include_str!("mock_intrinsic_measure.doc/_module.md")]

use std::collections::HashMap;

use obscur_geometry::{BoxConstraints, Size};
use obscur_identity::NodeId;

use crate::model::IntrinsicSize;

use super::{IntrinsicMeasure, LayoutEnvironment};

#[doc = include_str!("mock_intrinsic_measure.doc/MockIntrinsicMeasure.md")]
#[derive(Debug, Clone, Default)]
pub struct MockIntrinsicMeasure {
    natural_sizes: HashMap<NodeId, Size>,
    default_size: Size,
}

impl MockIntrinsicMeasure {
    #[doc = include_str!("mock_intrinsic_measure.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            natural_sizes: HashMap::new(),
            default_size: Size::ZERO,
        }
    }

    #[doc = include_str!("mock_intrinsic_measure.doc/with_default.md")]
    #[must_use]
    pub fn with_default(size: Size) -> Self {
        Self {
            natural_sizes: HashMap::new(),
            default_size: size,
        }
    }

    #[doc = include_str!("mock_intrinsic_measure.doc/set.md")]
    pub fn set(&mut self, node: NodeId, size: Size) -> &mut Self {
        self.natural_sizes.insert(node, size);
        self
    }

    #[doc = include_str!("mock_intrinsic_measure.doc/natural_size.md")]
    #[must_use]
    fn natural_size(&self, node: NodeId) -> Size {
        self.natural_sizes
            .get(&node)
            .copied()
            .unwrap_or(self.default_size)
    }
}

impl IntrinsicMeasure for MockIntrinsicMeasure {
    #[doc = include_str!("mock_intrinsic_measure.doc/measure.md")]
    fn measure(&self, node: NodeId, constraints: BoxConstraints, _env: &LayoutEnvironment) -> Size {
        constraints.constrain(self.natural_size(node))
    }

    #[doc = include_str!("mock_intrinsic_measure.doc/intrinsic_size.md")]
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
