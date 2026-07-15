//! A single sizing input for a layout node (width, height, flex-basis, ...).

/// A sizing input expressed in layout's own vocabulary (spec §4.1).
///
/// A [`Dimension`] describes how a length is specified before resolution:
/// automatic (content/context driven), an absolute number of logical pixels,
/// a percentage of some base length, or one of the **intrinsic** sizing
/// keywords, which are answered by measuring content rather than by arithmetic.
/// All values are fractional logical pixels; no rounding is applied here
/// (spec §4.6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    /// The length is determined by content/context rather than an explicit value.
    Auto,
    /// An absolute length, in logical pixels.
    Px(f32),
    /// A percentage (`0.0..=100.0` style, e.g. `50.0` means 50%) of a base length.
    Percent(f32),
    /// `min-content`: the smallest size the content can take without overflowing
    /// — in the inline axis, the widest unbreakable run (CSS Sizing 3 §4.1).
    MinContent,
    /// `max-content`: the size the content wants given unbounded space — in the
    /// inline axis, everything on one line (CSS Sizing 3 §4.1).
    MaxContent,
    /// `fit-content`: `min(max-content, max(min-content, stretch-fit))` — the
    /// classic shrink-to-fit (CSS Sizing 3 §4.1, CSS 2.1 §10.3.5).
    FitContent,
}

impl Dimension {
    /// Resolves this dimension against a `base` length, in logical pixels.
    ///
    /// * [`Dimension::Px`] resolves to its stored value.
    /// * [`Dimension::Percent`] resolves to `base * percent / 100.0` — but only
    ///   against a **definite** base; see below.
    /// * [`Dimension::Auto`] has no resolvable value and yields [`None`].
    /// * The intrinsic keywords yield [`None`]: they are not resolvable by
    ///   arithmetic at all, because the answer is a property of the content. An
    ///   algorithm that supports them matches on them *before* calling this and
    ///   measures; one that does not falls back to [`None`] and so treats them as
    ///   `auto` — see [`is_intrinsic`](Self::is_intrinsic).
    ///
    /// # An indefinite base makes a percentage `auto`
    ///
    /// `base` is not finite while content is being measured under unbounded space
    /// (a `max-content` probe offers an infinite inline axis, and a `height: auto`
    /// block offers an infinite block axis). CSS Sizing 3 §5.1: a percentage
    /// against an indefinite basis "behaves as `auto`". Resolving it arithmetically
    /// instead would return infinity and poison every size derived from it — a
    /// `max-content` box containing a `50%` child would come out infinitely wide.
    #[must_use]
    pub fn resolve(self, base: f32) -> Option<f32> {
        match self {
            Self::Auto | Self::MinContent | Self::MaxContent | Self::FitContent => None,
            Self::Px(value) => Some(value),
            Self::Percent(percent) => base.is_finite().then(|| base * percent / 100.0),
        }
    }

    /// Returns `true` if this dimension is [`Dimension::Auto`].
    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }

    /// Returns `true` for the intrinsic sizing keywords — the ones that can only
    /// be answered by measuring content.
    ///
    /// [`resolve`](Self::resolve) returns [`None`] for these *and* for
    /// [`Auto`](Self::Auto), which is deliberate (an algorithm with no intrinsic
    /// support degrades to its `auto` behaviour rather than misbehaving) but makes
    /// the two indistinguishable at that seam. This predicate is how an algorithm
    /// that *does* support them tells them apart.
    #[must_use]
    pub const fn is_intrinsic(self) -> bool {
        matches!(self, Self::MinContent | Self::MaxContent | Self::FitContent)
    }
}

#[cfg(test)]
mod tests {
    use super::Dimension;

    #[test]
    fn px_resolves_to_its_value() {
        assert_eq!(Dimension::Px(12.5).resolve(100.0), Some(12.5));
    }

    #[test]
    fn percent_resolves_against_base() {
        assert_eq!(Dimension::Percent(50.0).resolve(80.0), Some(40.0));
    }

    #[test]
    fn auto_never_resolves() {
        assert_eq!(Dimension::Auto.resolve(100.0), None);
        assert!(Dimension::Auto.is_auto());
        assert!(!Dimension::Px(0.0).is_auto());
    }

    #[test]
    fn an_intrinsic_keyword_has_no_arithmetic_answer() {
        // Not resolvable against a base at any size: the answer lives in the
        // content. An algorithm must measure, or fall back to `auto`.
        for dim in [
            Dimension::MinContent,
            Dimension::MaxContent,
            Dimension::FitContent,
        ] {
            assert_eq!(dim.resolve(100.0), None);
            assert!(dim.is_intrinsic());
            assert!(
                !dim.is_auto(),
                "intrinsic is not `auto`, even though both resolve to None"
            );
        }
        assert!(!Dimension::Auto.is_intrinsic());
        assert!(!Dimension::Px(1.0).is_intrinsic());
        assert!(!Dimension::Percent(50.0).is_intrinsic());
    }

    #[test]
    fn a_percentage_against_an_indefinite_base_behaves_as_auto() {
        // CSS Sizing 3 §5.1. This is what keeps a `max-content` probe — which
        // offers an infinite inline axis — from producing an infinitely wide box.
        assert_eq!(Dimension::Percent(50.0).resolve(f32::INFINITY), None);
        assert_eq!(Dimension::Percent(50.0).resolve(f32::NAN), None);
        // An absolute length is unmoved: it never needed the base.
        assert_eq!(Dimension::Px(12.0).resolve(f32::INFINITY), Some(12.0));
        // And a definite base still resolves exactly as before.
        assert_eq!(Dimension::Percent(50.0).resolve(80.0), Some(40.0));
    }
}
