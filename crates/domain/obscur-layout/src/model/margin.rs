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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Margin {
    /// An absolute margin, in logical pixels. May be negative.
    Px(f32),
    /// A percentage of the containing block's **inline** size, as a percentage
    /// value (`50.0` for `50%`).
    ///
    /// The inline size for *every* edge, including `margin-top` and
    /// `margin-bottom` — CSS 2.1 §8.3 resolves all four against the containing
    /// block's *width*, not each edge's own axis. It reads like a bug and is not:
    /// a block's height often depends on its content, so resolving a block-axis
    /// percentage against it would be circular. One axis is chosen to break the
    /// cycle, and CSS chose the inline one.
    Percent(f32),
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

    /// This edge as a plain length, against a containing-block inline size of
    /// `basis`.
    ///
    /// * `Px` is already a length.
    /// * `Percent` is `basis × percent / 100`. An `basis` that is not finite —
    ///   which is what a `max-content` probe offers — yields `0`: CSS Sizing 3
    ///   §5.2 treats a percentage margin as zero when computing an intrinsic
    ///   contribution, for the same reason it does for a percentage size (there is
    ///   nothing definite to be a percentage *of*). Passing `0.0` as the basis is
    ///   the honest way for a caller with no containing block to say so.
    /// * `Auto` is `0` — its used value whenever there is no leftover space to
    ///   absorb, which CSS 2.1 §10.3.3 makes the common case: a `width: auto` block
    ///   already fills its container.
    ///
    /// Use this wherever an edge is *consumed* as a length; use
    /// [`is_auto`](Self::is_auto) where the leftover actually gets distributed,
    /// because this collapses that question to `0`.
    #[must_use]
    pub fn resolve(self, basis: f32) -> f32 {
        match self {
            Self::Px(value) => value,
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
