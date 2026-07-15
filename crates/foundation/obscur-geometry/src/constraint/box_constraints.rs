//! Min/max box constraints for the measure pass.

use crate::primitive::Size;

/// Lower and upper bounds on a box's logical size, passed down during the
/// measure phase of layout (spec §4.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxConstraints {
    /// Minimum size.
    pub min: Size,
    /// Maximum size. Components may be [`f32::INFINITY`] for unbounded axes.
    pub max: Size,
}

impl BoxConstraints {
    /// Constraints that admit any size from zero up to `max`.
    #[must_use]
    pub const fn loose(max: Size) -> Self {
        Self {
            min: Size::ZERO,
            max,
        }
    }

    /// Constraints that force exactly `size`.
    #[must_use]
    pub const fn tight(size: Size) -> Self {
        Self {
            min: size,
            max: size,
        }
    }

    /// Clamps a desired size into these constraints.
    #[must_use]
    pub fn constrain(self, desired: Size) -> Size {
        Size::new(
            desired.width.clamp(self.min.width, self.max.width),
            desired.height.clamp(self.min.height, self.max.height),
        )
    }

    /// Resolves a desired size the way a box that is allowed to **overflow**
    /// should be sized: a *tight* axis dictates, a *loose* axis only offers.
    ///
    /// Per axis:
    ///
    /// * **tight** (`min == max`, as built by [`tight`](Self::tight)) — the caller
    ///   is not offering space, it is assigning a size. The desired value is
    ///   ignored and the assigned one is used. An absolutely-positioned box with
    ///   both insets set, and the root under the viewport, are sized this way.
    /// * **loose** (`min < max`, as built by [`loose`](Self::loose)) — `max` is the
    ///   space *available*, not a ceiling the content is obliged to fit. Content
    ///   that needs more reports more, and the result exceeds `max`. The value is
    ///   still floored at `min`.
    ///
    /// # Why this exists next to `constrain`
    ///
    /// [`constrain`](Self::constrain) caps at `max` unconditionally, which reports
    /// the space a box was *offered* rather than the size it actually took. For
    /// content that fits, the two agree. For content that does not, `constrain`
    /// answers with a number that was never true — and the caller cannot tell,
    /// because overflow and exact-fit come back identical. That matters most for
    /// intrinsic sizing, where the whole question is "how much does this need?":
    /// a `min-content` probe offers zero space, and under `constrain` every answer
    /// would be zero.
    ///
    /// `constrain` remains right wherever the caller genuinely means "clip this
    /// into range" rather than "size this box".
    ///
    /// # A zero axis reads as an offer, not an assignment
    ///
    /// `loose(Size::ZERO)` and `tight(Size::ZERO)` are the *same value* — both are
    /// `min == max == 0` — so at zero these two intents are genuinely
    /// indistinguishable in this model, and this method has to pick one. It picks
    /// **offer**: a zero axis means "there is no space here", and the box reports
    /// what it needs.
    ///
    /// That is the reading with something real behind it. "You have no space" is
    /// the `min-content` probe, which must work; "you must be exactly zero" is a
    /// degenerate assignment (an absolutely-positioned box stretched across a
    /// zero-wide containing block), and answering it with the content's own size
    /// costs nothing that was not already collapsed. Reading it the other way would
    /// silently make every `min-content` answer zero.
    ///
    /// Telling the two apart for real needs tightness carried explicitly rather
    /// than inferred from the numbers — a change to this type, not to this method.
    #[must_use]
    pub fn constrain_allowing_overflow(self, desired: Size) -> Size {
        Size::new(
            resolve_axis(desired.width, self.min.width, self.max.width),
            resolve_axis(desired.height, self.min.height, self.max.height),
        )
    }
}

/// One axis of [`BoxConstraints::constrain_allowing_overflow`].
///
/// `min >= max` covers the tight case (`min == max`) and, defensively, an
/// over-constrained one — in both the bounds leave no choice to make, so the upper
/// bound is used rather than propagating a contradiction. `max > 0` keeps a zero
/// axis out of that branch: see the note on
/// [`constrain_allowing_overflow`](BoxConstraints::constrain_allowing_overflow).
fn resolve_axis(desired: f32, min: f32, max: f32) -> f32 {
    if min >= max && max > 0.0 {
        max
    } else {
        desired.max(min)
    }
}

#[cfg(test)]
mod tests {
    use super::BoxConstraints;
    use crate::primitive::Size;

    #[test]
    fn a_tight_axis_dictates_and_ignores_what_the_box_wanted() {
        // How an absolutely-positioned box with both insets, and the root under
        // the viewport, are sized: the caller is assigning, not offering.
        let c = BoxConstraints::tight(Size::new(260.0, 300.0));
        assert_eq!(
            c.constrain_allowing_overflow(Size::new(999.0, 999.0)),
            Size::new(260.0, 300.0)
        );
        assert_eq!(
            c.constrain_allowing_overflow(Size::new(1.0, 1.0)),
            Size::new(260.0, 300.0)
        );
    }

    #[test]
    fn a_loose_axis_lets_content_overflow_the_space_offered() {
        // THE difference from `constrain`: 86.4 of unbreakable content offered
        // 40px reports 86.4, because that is what it needs.
        let c = BoxConstraints::loose(Size::new(40.0, 400.0));
        let got = c.constrain_allowing_overflow(Size::new(86.4, 10.0));
        assert_eq!(got.width, 86.4);
        assert_eq!(
            c.constrain(Size::new(86.4, 10.0)).width,
            40.0,
            "`constrain` reports the space offered — the answer that was never true"
        );
    }

    #[test]
    fn a_loose_axis_still_floors_at_the_minimum() {
        let c = BoxConstraints {
            min: Size::new(50.0, 5.0),
            max: Size::new(400.0, 400.0),
        };
        assert_eq!(
            c.constrain_allowing_overflow(Size::new(10.0, 1.0)),
            Size::new(50.0, 5.0)
        );
    }

    #[test]
    fn content_that_fits_is_unaffected() {
        // The overwhelmingly common case: the two agree exactly, so nothing that
        // fits its container changes behaviour.
        let c = BoxConstraints::loose(Size::new(400.0, 400.0));
        let desired = Size::new(120.0, 33.0);
        assert_eq!(c.constrain_allowing_overflow(desired), desired);
        assert_eq!(c.constrain_allowing_overflow(desired), c.constrain(desired));
    }

    #[test]
    fn a_zero_offer_lets_content_report_what_it_needs() {
        // The `min-content` probe. `loose(0)` and `tight(0)` are the same value,
        // so this axis is read as "no space available" rather than "be zero" —
        // otherwise every min-content answer would be zero. See the method's note.
        let c = BoxConstraints::loose(Size::new(0.0, 400.0));
        assert_eq!(
            c.constrain_allowing_overflow(Size::new(86.4, 48.0)),
            Size::new(86.4, 48.0),
            "a zero offer must not be mistaken for a zero assignment"
        );
    }

    #[test]
    fn an_unbounded_axis_returns_what_was_wanted() {
        // A `max-content` probe: infinite space offered, a finite want reported.
        let c = BoxConstraints::loose(Size::new(f32::INFINITY, f32::INFINITY));
        assert_eq!(
            c.constrain_allowing_overflow(Size::new(201.6, 16.0)),
            Size::new(201.6, 16.0)
        );
    }
}
