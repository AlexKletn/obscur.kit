//! FixedChildMeasurer: a deterministic [`ChildMeasurer`] for tests.

use obscur_geometry::{BoxConstraints, Size};

use super::ChildMeasurer;

/// A [`ChildMeasurer`] backed by a fixed natural size per child index.
///
/// [`ChildMeasurer`]: super::ChildMeasurer
///
/// This exists so layout algorithms can be exercised in isolation — without a
/// runtime, a retained tree, or the intrinsic-measure port. Each child index is
/// assigned one *natural* size; a measurement returns that natural size clamped
/// into the requested constraints (via [`BoxConstraints::constrain`]). An index
/// outside the backing vector measures as [`Size::ZERO`], so the measurer never
/// panics regardless of the indices an algorithm asks for.
///
/// It is deterministic and side-effect-free: repeated calls with the same
/// `(index, constraints)` always return the same size.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FixedChildMeasurer {
    sizes: Vec<Size>,
}

impl FixedChildMeasurer {
    /// Creates a measurer from one natural size per child index.
    ///
    /// `sizes[i]` is the natural (unconstrained) size reported for child `i`.
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
    /// Returns the natural size for `index`, clamped into `constraints`.
    ///
    /// Out-of-range indices measure as [`Size::ZERO`] (still clamped into
    /// `constraints`, so a non-zero minimum is honoured).
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
