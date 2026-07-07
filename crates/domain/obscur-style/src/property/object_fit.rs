//! The computed value of CSS `object-fit` (CSS Images §5.5) — the replaced-element
//! analog of `background-size`.

/// The computed value of `object-fit`: how a replaced element's content (e.g. an
/// `<img>`'s decoded pixels) is scaled to fit its content box.
///
/// The scene coordinator turns this into a concrete tile rect within the element's
/// box at paint time, reusing the same cover/contain/fit geometry as
/// `background-size`:
///
/// * [`ObjectFit::Fill`] — stretch to exactly fill the box, ignoring the intrinsic
///   aspect ratio (the CSS replaced-element default).
/// * [`ObjectFit::Contain`] — scale to the largest size that fits inside the box,
///   preserving the aspect ratio (letterboxed).
/// * [`ObjectFit::Cover`] — scale to the smallest size that fully covers the box,
///   preserving the aspect ratio (overflow clipped to the box).
/// * [`ObjectFit::None`] — draw at the intrinsic pixel size, unscaled.
/// * [`ObjectFit::ScaleDown`] — the smaller of [`ObjectFit::None`] and
///   [`ObjectFit::Contain`]: like `none`, but shrunk to fit when the intrinsic size
///   is larger than the box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ObjectFit {
    /// Stretch to fill the box, ignoring the aspect ratio (the CSS initial).
    #[default]
    Fill,
    /// Scale to fit inside the box, aspect ratio preserved.
    Contain,
    /// Scale to cover the box, aspect ratio preserved (overflow clipped).
    Cover,
    /// Draw at the intrinsic pixel size, unscaled.
    None,
    /// The smaller of `none` and `contain`.
    ScaleDown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_fill() {
        assert_eq!(ObjectFit::default(), ObjectFit::Fill);
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(ObjectFit::Fill, ObjectFit::Contain);
        assert_ne!(ObjectFit::Cover, ObjectFit::None);
        assert_ne!(ObjectFit::None, ObjectFit::ScaleDown);
    }
}
