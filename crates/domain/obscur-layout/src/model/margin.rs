//! One margin edge, which is a length or `auto`.

/// A single margin edge (spec §4.2, CSS 2.1 §8.3).
///
/// A margin is a length in logical pixels or the `auto` keyword — and `auto` is
/// not a number that can be substituted early: it means *"take what is left
/// over"*, and how much that is depends on the box's resolved size, which is not
/// known until it has been measured. `margin: 0 auto` centring is exactly that
/// rule doing its job.
///
/// # Why this is not `Dimension`
///
/// [`Dimension`](super::Dimension) already models `Auto` / `Px` / `Percent`, and
/// reusing it here would be a smaller diff. It would also let a caller *say*
/// `margin-left: max-content`, which is not a thing: since intrinsic sizing landed,
/// `Dimension` carries `MinContent` / `MaxContent` / `FitContent`, and a type that
/// can express nonsense pushes the job of rejecting it onto every reader. This
/// enum says exactly what a margin can be, so there is nothing to reject.
///
/// `Percent` is absent for the same reason: percentage margins are not resolved by
/// this engine (they never were — the CSS bridge folded them to `0`). When they
/// are, they arrive as a variant here and every `match` stops compiling until it
/// has an answer, which is the point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Margin {
    /// An absolute margin, in logical pixels. May be negative.
    Px(f32),
    /// The `auto` keyword: this edge absorbs leftover space in its axis.
    Auto,
}

impl Margin {
    /// A zero margin — the initial value of every margin edge.
    pub const ZERO: Self = Self::Px(0.0);

    /// Returns `true` when this edge is `auto`.
    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }

    /// This edge's contribution as a plain length, treating `auto` as `0`.
    ///
    /// `0` is what `auto` *used* value is whenever there is no leftover space to
    /// absorb — which CSS 2.1 §10.3.3 makes the common case: a `width: auto` block
    /// already fills its container, so its `auto` margins resolve to zero. Use this
    /// wherever an edge is being consumed as a length; use
    /// [`is_auto`](Self::is_auto) where the leftover actually gets distributed.
    #[must_use]
    pub const fn or_zero(self) -> f32 {
        match self {
            Self::Px(value) => value,
            Self::Auto => 0.0,
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
        assert_eq!(Margin::Auto.or_zero(), 0.0);
    }

    #[test]
    fn a_length_edge_keeps_its_value_including_a_negative_one() {
        assert_eq!(Margin::Px(12.5).or_zero(), 12.5);
        assert_eq!(Margin::Px(-8.0).or_zero(), -8.0, "negative margins are legal CSS");
        assert!(!Margin::Px(-8.0).is_auto());
    }
}
