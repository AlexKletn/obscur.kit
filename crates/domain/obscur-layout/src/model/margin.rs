//! One margin edge, which is a length or `auto`.

use std::sync::Arc;

use super::TrackLimit;

#[doc = include_str!("../../docs/api/model/margin.doc/Margin.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum Margin {
    /// An absolute margin, in logical pixels. May be negative.
    Px(f32),
    #[doc = include_str!("../../docs/api/model/margin.doc/Percent.md")]
    Percent(f32),
    /// A signed margin expression resolved against the containing inline extent.
    Math(Arc<TrackLimit>),
    /// The `auto` keyword: this edge absorbs leftover space in its axis.
    Auto,
}

impl Margin {
    /// A zero margin — the initial value of every margin edge.
    pub const ZERO: Self = Self::Px(0.0);

    /// Returns `true` when this edge is `auto`.
    #[must_use]
    pub const fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    #[doc = include_str!("../../docs/api/model/margin.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, basis: f32) -> f32 {
        match self {
            Self::Px(value) => *value,
            Self::Math(expression) => expression
                .resolve(if basis.is_finite() { basis } else { 0.0 })
                .unwrap_or(0.0),
            Self::Percent(percent) if basis.is_finite() => basis * percent / 100.0,
            Self::Percent(_) | Self::Auto => 0.0,
        }
    }
}

impl Default for Margin {
    fn default() -> Self {
        Self::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::Margin;

    #[test]
    fn zero_is_the_default_and_a_length() {
        assert_eq!(Margin::default(), Margin::ZERO);
        assert_eq!(Margin::ZERO, Margin::Px(0.0));
        assert!(!Margin::ZERO.is_auto());
    }

    #[test]
    fn auto_reads_as_zero_when_consumed_as_a_length() {
        assert!(Margin::Auto.is_auto());
        assert_eq!(Margin::Auto.resolve(500.0), 0.0);
    }

    #[test]
    fn a_length_edge_keeps_its_value_including_a_negative_one() {
        assert_eq!(Margin::Px(12.5).resolve(500.0), 12.5);
        assert_eq!(
            Margin::Px(-8.0).resolve(500.0),
            -8.0,
            "negative margins are legal CSS"
        );
        assert!(!Margin::Px(-8.0).is_auto());
        // The basis is irrelevant to a length — including an indefinite one.
        assert_eq!(Margin::Px(12.5).resolve(f32::INFINITY), 12.5);
    }

    #[test]
    fn a_percentage_resolves_against_the_basis() {
        assert_eq!(Margin::Percent(10.0).resolve(500.0), 50.0);
        assert_eq!(Margin::Percent(50.0).resolve(80.0), 40.0);
        assert!(!Margin::Percent(50.0).is_auto(), "a `%` is not `auto`");
    }

    #[test]
    fn a_percentage_against_an_indefinite_basis_is_zero() {
        // What a `max-content` probe offers. CSS Sizing 3 §5.2: a percentage
        // margin contributes zero to an intrinsic size, because there is nothing
        // definite for it to be a percentage of. Resolving it arithmetically would
        // return infinity and take the whole box with it.
        assert_eq!(Margin::Percent(50.0).resolve(f32::INFINITY), 0.0);
        assert_eq!(Margin::Percent(50.0).resolve(f32::NAN), 0.0);
        // And a caller with no containing block says so by passing a zero basis.
        assert_eq!(Margin::Percent(50.0).resolve(0.0), 0.0);
    }
}
