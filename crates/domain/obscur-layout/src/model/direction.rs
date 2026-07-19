//! The CSS `direction` model: the inline base direction of a box's content.

/// The inline base direction a box lays its in-flow content out along
/// (CSS `direction`, spec §7.3).
///
/// This is the **containing block's** property as far as layout is concerned:
/// css-writing-modes-4 §7.2 ("Abstract Box Terminology") says flow-relative
/// directions "are calculated with respect to the writing mode of the containing
/// block of the box", and gives the over-constrained margin as its worked
/// example — "the margin that is dropped when a box's inline dimension is
/// over-constrained is the end margin as determined by the writing mode of the
/// containing block".
///
/// * [`Ltr`](Self::Ltr) — the CSS initial. `inline-start` is the line-left side,
///   so an over-constrained in-flow child keeps its `margin-left` and drops
///   `margin-right`.
/// * [`Rtl`](Self::Rtl) — `inline-start` is the line-right side
///   (css-writing-modes-4 §6.2), so an over-constrained in-flow child keeps its
///   `margin-right` and drops `margin-left`, which lands it against the
///   line-right edge of the content box.
///
/// # What this does *not* cover
///
/// This value carries the *box-model* half of `direction` only: which physical
/// edge `inline-start` names when a block-level box is placed in normal flow.
/// Bidirectional text reordering (UAX #9) is a separate mechanism in the text
/// stack and does not read this type; neither does flex or grid layout, whose
/// own inline-axis reversal is not modelled here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Left-to-right inline base direction (the CSS initial value).
    #[default]
    Ltr,
    /// Right-to-left inline base direction (`direction: rtl`).
    Rtl,
}

impl Direction {
    /// Whether this direction is [`Rtl`](Self::Rtl).
    #[must_use]
    pub const fn is_rtl(self) -> bool {
        matches!(self, Self::Rtl)
    }
}

#[cfg(test)]
mod tests {
    use super::Direction;

    #[test]
    fn default_is_ltr() {
        assert_eq!(Direction::default(), Direction::Ltr);
        assert!(!Direction::default().is_rtl());
    }

    #[test]
    fn rtl_reports_rtl() {
        assert!(Direction::Rtl.is_rtl());
    }
}
