#![doc = include_str!("dimension.doc/_module.md")]

#[doc = include_str!("dimension.doc/Dimension.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    #[doc = include_str!("dimension.doc/Auto.md")]
    Auto,
    #[doc = include_str!("dimension.doc/Px.md")]
    Px(f32),
    #[doc = include_str!("dimension.doc/Percent.md")]
    Percent(f32),
    #[doc = include_str!("dimension.doc/MinContent.md")]
    MinContent,
    #[doc = include_str!("dimension.doc/MaxContent.md")]
    MaxContent,
    #[doc = include_str!("dimension.doc/FitContent.md")]
    FitContent,
}

impl Dimension {
    #[doc = include_str!("dimension.doc/resolve.md")]
    #[must_use]
    pub fn resolve(self, base: f32) -> Option<f32> {
        match self {
            Self::Auto | Self::MinContent | Self::MaxContent | Self::FitContent => None,
            Self::Px(value) => Some(value),
            Self::Percent(percent) => base.is_finite().then(|| base * percent / 100.0),
        }
    }

    #[doc = include_str!("dimension.doc/is_auto.md")]
    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }

    #[doc = include_str!("dimension.doc/is_intrinsic.md")]
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
