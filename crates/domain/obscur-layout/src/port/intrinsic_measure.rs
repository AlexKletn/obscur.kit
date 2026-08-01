#![doc = include_str!("intrinsic_measure.doc/_module.md")]

use obscur_geometry::{BoxConstraints, Size};
use obscur_identity::NodeId;

use crate::model::IntrinsicSize;

use super::LayoutEnvironment;

#[doc = include_str!("intrinsic_measure.doc/IntrinsicMeasure.md")]
pub trait IntrinsicMeasure {
    #[doc = include_str!("intrinsic_measure.doc/measure.md")]
    fn measure(&self, node: NodeId, constraints: BoxConstraints, env: &LayoutEnvironment) -> Size;

    #[doc = include_str!("intrinsic_measure.doc/intrinsic_size.md")]
    fn intrinsic_size(&self, node: NodeId, env: &LayoutEnvironment) -> IntrinsicSize;
}

#[cfg(test)]
mod tests {
    use super::IntrinsicMeasure;
    use crate::model::IntrinsicSize;
    use crate::port::LayoutEnvironment;
    use obscur_geometry::{BoxConstraints, Size};
    use obscur_identity::NodeId;

    /// A trivial in-test implementation confirming the trait is object-usable
    /// and its methods have the expected shapes.
    struct Fixed(Size);

    impl IntrinsicMeasure for Fixed {
        fn measure(
            &self,
            _node: NodeId,
            constraints: BoxConstraints,
            _env: &LayoutEnvironment,
        ) -> Size {
            constraints.constrain(self.0)
        }

        fn intrinsic_size(&self, _node: NodeId, _env: &LayoutEnvironment) -> IntrinsicSize {
            IntrinsicSize::fixed(self.0)
        }
    }

    #[test]
    fn trait_is_callable() {
        let provider = Fixed(Size::new(30.0, 10.0));
        let env = LayoutEnvironment::identity();
        let node = NodeId::from_raw(1);
        assert_eq!(
            provider.measure(node, BoxConstraints::loose(Size::new(100.0, 100.0)), &env),
            Size::new(30.0, 10.0)
        );
        assert_eq!(
            provider.intrinsic_size(node, &env),
            IntrinsicSize::fixed(Size::new(30.0, 10.0))
        );
    }
}
