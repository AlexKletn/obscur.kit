//! CSS length expressions resolved against a layout axis.

use crate::model::Dimension;

/// A length expression with leaves resolved to pixels or percentages.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackLimit {
    /// A resolved length or a percentage of the grid axis.
    Length(Dimension),
    /// Addition.
    Sum(Box<Self>, Box<Self>),
    /// Subtraction.
    Difference(Box<Self>, Box<Self>),
    /// Multiplication by a scalar.
    Scale(Box<Self>, f32),
    /// Division by a scalar.
    Quotient(Box<Self>, f32),
    /// The least of the terms.
    Min(Vec<Self>),
    /// The greatest of the terms.
    Max(Vec<Self>),
    /// A value constrained to an interval, with precedence given to the minimum.
    Clamp(Box<Self>, Box<Self>, Box<Self>),
}

impl TrackLimit {
    /// Reports whether evaluation requires a definite grid axis.
    pub(crate) fn uses_percentage(&self) -> bool {
        match self {
            Self::Length(dimension) => dimension.uses_percentage(),
            Self::Sum(a, b) | Self::Difference(a, b) => a.uses_percentage() || b.uses_percentage(),
            Self::Scale(value, _) | Self::Quotient(value, _) => value.uses_percentage(),
            Self::Min(terms) | Self::Max(terms) => terms.iter().any(Self::uses_percentage),
            Self::Clamp(a, b, c) => {
                a.uses_percentage() || b.uses_percentage() || c.uses_percentage()
            }
        }
    }

    /// Evaluates the cap with the supplied percentage basis.
    pub(crate) fn resolve(&self, basis: f32) -> Option<f32> {
        let value = self.resolve_wide(basis)?;
        (!value.is_nan()).then(|| obscur_geometry::saturating_f32(value))
    }

    /// Evaluates track math before narrowing intermediate results.
    pub(super) fn resolve_wide(&self, basis: f32) -> Option<f64> {
        let resolve = |term: &Self| term.resolve_wide(basis);
        let combine = |terms: &[Self], op: fn(f64, f64) -> f64| {
            let mut terms = terms.iter().map(resolve);
            terms
                .next()?
                .and_then(|first| terms.try_fold(first, |value, term| Some(op(value, term?))))
        };
        match self {
            Self::Length(dimension) => dimension.resolve_wide(basis),
            Self::Sum(a, b) => Some(resolve(a)? + resolve(b)?),
            Self::Difference(a, b) => Some(resolve(a)? - resolve(b)?),
            Self::Scale(value, scalar) => Some(resolve(value)? * f64::from(*scalar)),
            Self::Quotient(value, scalar) if *scalar != 0.0 => {
                Some(resolve(value)? / f64::from(*scalar))
            }
            Self::Quotient(..) => None,
            Self::Min(terms) => combine(terms, f64::min),
            Self::Max(terms) => combine(terms, f64::max),
            Self::Clamp(min, value, max) => {
                Some(resolve(min)?.max(resolve(value)?.min(resolve(max)?)))
            }
        }
    }
}
