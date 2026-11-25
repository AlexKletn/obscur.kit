//! The measure-phase input for a node.

use obscur_geometry::{BoxConstraints, Size};

#[doc = include_str!("../../docs/api/pass/measure_pass.doc/MeasurePass.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeasurePass {
    /// The constraints the node is measured against.
    constraints: BoxConstraints,
}

impl MeasurePass {
    /// Creates a measure input carrying the given `constraints`.
    #[must_use]
    pub const fn new(constraints: BoxConstraints) -> Self {
        Self { constraints }
    }

    /// The constraints the node is measured against.
    #[must_use]
    pub const fn constraints(self) -> BoxConstraints {
        self.constraints
    }

    /// Clamps a `desired` size into these constraints (delegates to
    /// [`BoxConstraints::constrain`]).
    #[must_use]
    pub fn constrain(self, desired: Size) -> Size {
        self.constraints.constrain(desired)
    }
}

#[cfg(test)]
mod tests {
    use super::MeasurePass;
    use obscur_geometry::{BoxConstraints, Size};

    #[test]
    fn exposes_constraints() {
        let constraints = BoxConstraints::loose(Size::new(100.0, 50.0));
        let pass = MeasurePass::new(constraints);
        assert_eq!(pass.constraints(), constraints);
    }

    #[test]
    fn constrain_clamps_desired_into_constraints() {
        let pass = MeasurePass::new(BoxConstraints::loose(Size::new(100.0, 50.0)));
        assert_eq!(
            pass.constrain(Size::new(200.0, 30.0)),
            Size::new(100.0, 30.0)
        );
    }

    #[test]
    fn tight_constraints_force_exact_size() {
        let pass = MeasurePass::new(BoxConstraints::tight(Size::new(40.0, 20.0)));
        assert_eq!(pass.constrain(Size::new(999.0, 1.0)), Size::new(40.0, 20.0));
    }
}
