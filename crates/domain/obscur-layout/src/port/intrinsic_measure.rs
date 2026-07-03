//! IntrinsicMeasure: the narrow port through which layout measures content.

use obscur_geometry::{BoxConstraints, Size};
use obscur_identity::NodeId;

use crate::model::IntrinsicSize;

use super::LayoutEnvironment;

/// The narrow **port** layout calls to measure content it cannot size itself —
/// text runs and replaced elements such as images or canvases (spec §2.3).
///
/// Layout never shapes text or decodes images; instead it asks a provider
/// behind this port. That keeps the layout engine independent of text and
/// image-decoding internals: the real text engine lives in `obscur-text` (added
/// in a later phase) and provides an adapter implementing this trait, while a
/// deterministic mock — [`MockIntrinsicMeasure`](super::MockIntrinsicMeasure) —
/// serves the Phase-1 headless slice (spec §5.13).
///
/// Implementations **must** be deterministic and side-effect-free: the same
/// `(node, constraints, env)` inputs must always yield the same size, and a
/// measurement must never mutate observable state. This lets layout cache and
/// re-run measurement freely.
pub trait IntrinsicMeasure {
    /// Measures the preferred size of `node`'s content under `constraints`,
    /// within the ambient [`LayoutEnvironment`].
    ///
    /// The returned size should already lie within `constraints`; layout will
    /// nonetheless [`BoxConstraints::constrain`] it defensively, so a provider
    /// need not treat the bounds as an exact contract.
    fn measure(&self, node: NodeId, constraints: BoxConstraints, env: &LayoutEnvironment) -> Size;

    /// Returns the min-content / max-content [`IntrinsicSize`] of `node`'s
    /// content (spec §4.1), used by intrinsic sizing.
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
