//! The outcome of propagating a layout change upward toward a parent context.

use crate::pass::LayoutPassResult;

/// The outcome of propagating a layout change **upward** from a context toward
/// its parent (spec §4.3).
///
/// §4.3 propagates invalidation upward *only when a context's output geometry
/// changes*: an unchanged result cannot affect ancestors and stops there.
/// Independently, a **containment boundary** halts upward propagation even when
/// geometry changed, because contained layout cannot escape the boundary to
/// affect ancestors.
///
/// This value pairs those two facts so a caller can decide, via
/// [`continues_upward`](Self::continues_upward), whether to keep walking toward
/// the parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropagationResult {
    /// Whether this context's output geometry changed in the pass.
    geometry_changed: bool,
    /// Whether this context is a containment boundary that stops escape.
    contained: bool,
}

impl PropagationResult {
    /// Creates a result from whether geometry changed and whether this context
    /// is a containment boundary.
    #[must_use]
    pub const fn new(geometry_changed: bool, contained: bool) -> Self {
        Self {
            geometry_changed,
            contained,
        }
    }

    /// Builds a result from a completed [`LayoutPassResult`], reading its
    /// `geometry_changed()` flag, combined with whether this context contains
    /// layout.
    #[must_use]
    pub const fn from_pass(pass: &LayoutPassResult, contained: bool) -> Self {
        Self::new(pass.geometry_changed(), contained)
    }

    /// Whether this context's output geometry changed.
    #[must_use]
    pub const fn geometry_changed(&self) -> bool {
        self.geometry_changed
    }

    /// Whether this context is a containment boundary that stops escape.
    #[must_use]
    pub const fn contained(&self) -> bool {
        self.contained
    }

    /// Whether propagation should continue **upward** to the parent context.
    ///
    /// Per §4.3, propagation continues only when the output geometry changed
    /// (`geometry_changed`) **and** this context does not contain layout
    /// (`!contained`). A containment boundary halts propagation even on a
    /// geometry change, and stable geometry halts it regardless of containment.
    #[must_use]
    pub const fn continues_upward(&self) -> bool {
        self.geometry_changed && !self.contained
    }
}

#[cfg(test)]
mod tests {
    use super::PropagationResult;
    use crate::model::LayoutBox;
    use crate::pass::LayoutPassResult;
    use obscur_geometry::Size;

    fn a_box() -> LayoutBox {
        LayoutBox::from_size(Size::new(10.0, 10.0))
    }

    #[test]
    fn continues_only_when_changed_and_not_contained() {
        assert!(PropagationResult::new(true, false).continues_upward());
        assert!(!PropagationResult::new(false, false).continues_upward());
        assert!(!PropagationResult::new(true, true).continues_upward());
        assert!(!PropagationResult::new(false, true).continues_upward());
    }

    #[test]
    fn from_pass_reads_geometry_flag() {
        let changed = LayoutPassResult::changed(a_box());
        let unchanged = LayoutPassResult::unchanged(a_box());

        let r = PropagationResult::from_pass(&changed, false);
        assert!(r.geometry_changed());
        assert!(r.continues_upward());

        let r = PropagationResult::from_pass(&unchanged, false);
        assert!(!r.geometry_changed());
        assert!(!r.continues_upward());
    }

    #[test]
    fn from_pass_respects_containment() {
        let changed = LayoutPassResult::changed(a_box());
        let r = PropagationResult::from_pass(&changed, true);
        assert!(r.geometry_changed());
        assert!(r.contained());
        assert!(!r.continues_upward());
    }
}
