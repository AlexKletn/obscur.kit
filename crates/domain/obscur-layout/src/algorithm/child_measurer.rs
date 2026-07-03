//! ChildMeasurer: the callback a layout algorithm uses to size a child.

use obscur_geometry::BoxConstraints;
use obscur_geometry::Size;

/// The callback through which a [`LayoutAlgorithm`] measures one of a node's
/// children under chosen constraints, without knowing what the child *is*.
///
/// [`LayoutAlgorithm`]: super::LayoutAlgorithm
///
/// Layout algorithms in this crate do not recurse into the retained tree; the
/// runtime owns tree traversal (spec §4.2). When an algorithm needs a child's
/// size under some constraints, it calls [`measure_child`](Self::measure_child)
/// with the child's index (into the `children` slice the algorithm was given).
/// The runtime's implementation decides how to satisfy the request:
///
/// * for a **leaf** (text run or replaced element such as an image), it routes
///   to the intrinsic-measure port
///   ([`IntrinsicMeasure`](crate::port::IntrinsicMeasure));
/// * for a **container**, it recursively lays the child out under the given
///   constraints and returns the resulting border-box size.
///
/// Either way the algorithm sees only a `Size`, which keeps it independent of
/// text shaping, image decoding, and nested-context recursion.
///
/// Implementations should be deterministic: the same `(index, constraints)`
/// should yield the same `Size` within a single layout pass so an algorithm may
/// measure a child more than once (e.g. once for its natural main size, once for
/// its cross size) and reason about the results consistently.
pub trait ChildMeasurer {
    /// Measures child `index` under `constraints`, returning its border-box size.
    ///
    /// `index` refers to the position of the child within the `children` slice
    /// passed to the algorithm. The returned size lies within `constraints` when
    /// the implementation honours them; algorithms treat it as authoritative and
    /// do not re-clamp it.
    fn measure_child(&mut self, index: usize, constraints: BoxConstraints) -> Size;
}

#[cfg(test)]
mod tests {
    use super::ChildMeasurer;
    use obscur_geometry::{BoxConstraints, Size};

    /// A trivial in-test implementation confirming the trait is object-safe and
    /// usable behind `&mut dyn`.
    struct Constant(Size);

    impl ChildMeasurer for Constant {
        fn measure_child(&mut self, _index: usize, constraints: BoxConstraints) -> Size {
            constraints.constrain(self.0)
        }
    }

    #[test]
    fn callable_through_dyn() {
        let mut m = Constant(Size::new(40.0, 10.0));
        let dynamic: &mut dyn ChildMeasurer = &mut m;
        let size = dynamic.measure_child(0, BoxConstraints::loose(Size::new(100.0, 100.0)));
        assert_eq!(size, Size::new(40.0, 10.0));
    }
}
