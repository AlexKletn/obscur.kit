//! The computed value of CSS `object-fit` (CSS Images §5.5) — the replaced-element
//! analog of `background-size`.

#[doc = include_str!("../../docs/api/property/object_fit.doc/ObjectFit.md")]
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
