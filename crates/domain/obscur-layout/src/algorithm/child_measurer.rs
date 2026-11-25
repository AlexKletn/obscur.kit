//! `ChildMeasurer`: the callback a layout algorithm uses to size a child.

use obscur_geometry::BoxConstraints;
use obscur_geometry::Size;

#[doc = include_str!("../../docs/api/algorithm/child_measurer.doc/ChildMeasurer.md")]
pub trait ChildMeasurer {
    #[doc = include_str!("../../docs/api/algorithm/child_measurer.doc/measure_child.md")]
    fn measure_child(&mut self, index: usize, constraints: BoxConstraints) -> Size;

    /// Measures the child and returns its first baseline, when available, relative to its border-box top.
    fn measure_child_with_baseline(
        &mut self,
        index: usize,
        constraints: BoxConstraints,
    ) -> (Size, Option<f32>) {
        (self.measure_child(index, constraints), None)
    }

    /// Measures the child and returns its last baseline relative to the top of its border box.
    fn measure_child_with_last_baseline(
        &mut self,
        index: usize,
        constraints: BoxConstraints,
    ) -> (Size, Option<f32>) {
        (self.measure_child(index, constraints), None)
    }
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
