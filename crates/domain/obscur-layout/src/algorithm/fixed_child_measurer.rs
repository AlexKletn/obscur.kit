//! `FixedChildMeasurer`: a deterministic [`ChildMeasurer`] for tests.

use obscur_geometry::{BoxConstraints, Size};

use super::ChildMeasurer;

#[doc = include_str!("../../docs/api/algorithm/fixed_child_measurer.doc/FixedChildMeasurer.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FixedChildMeasurer {
    sizes: Vec<Size>,
}

impl FixedChildMeasurer {
    #[doc = include_str!("../../docs/api/algorithm/fixed_child_measurer.doc/new.md")]
    #[must_use]
    pub fn new(sizes: Vec<Size>) -> Self {
        Self { sizes }
    }

    /// The natural size registered for `index`, or [`Size::ZERO`] if out of range.
    #[must_use]
    pub fn natural(&self, index: usize) -> Size {
        self.sizes.get(index).copied().unwrap_or(Size::ZERO)
    }
}

impl ChildMeasurer for FixedChildMeasurer {
    #[doc = include_str!("../../docs/api/algorithm/fixed_child_measurer.doc/measure_child.md")]
    fn measure_child(&mut self, index: usize, constraints: BoxConstraints) -> Size {
        constraints.constrain(self.natural(index))
    }
}

#[cfg(test)]
mod tests {
    use super::{ChildMeasurer, FixedChildMeasurer};
    use obscur_geometry::{BoxConstraints, Size};

    #[test]
    fn returns_natural_clamped_into_constraints() {
        let mut m = FixedChildMeasurer::new(vec![Size::new(50.0, 20.0)]);
        // Loose beyond the natural size: natural is returned unchanged.
        assert_eq!(
            m.measure_child(0, BoxConstraints::loose(Size::new(100.0, 100.0))),
            Size::new(50.0, 20.0)
        );
        // Tight below the natural size: clamped down.
        assert_eq!(
            m.measure_child(0, BoxConstraints::tight(Size::new(10.0, 5.0))),
            Size::new(10.0, 5.0)
        );
    }

    #[test]
    fn out_of_range_index_is_zero() {
        let mut m = FixedChildMeasurer::new(vec![Size::new(3.0, 3.0)]);
        assert_eq!(
            m.measure_child(9, BoxConstraints::loose(Size::new(100.0, 100.0))),
            Size::ZERO
        );
        assert_eq!(m.natural(9), Size::ZERO);
    }
}
