#[doc = include_str!("../../docs/api/error/unsupported_css_feature.doc/UnsupportedFeatureKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnsupportedFeatureKind {
    /// An at-rule such as `@supports`.
    AtRule,
    /// A property such as `grid-template-areas`.
    Property,
    /// A selector construct.
    Selector,
    /// A value function such as `clip-path()`.
    Function,
    /// A unit the kit does not model.
    Unit,
    /// A pseudo-element such as `::backdrop`.
    PseudoElement,
    #[doc = include_str!("../../docs/api/error/unsupported_css_feature.doc/VendorProperty.md")]
    VendorProperty,
}

impl UnsupportedFeatureKind {
    /// Returns a short, stable label for this kind, used in
    /// [`UnsupportedCssFeature`]'s [`std::fmt::Display`] output.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::AtRule => "unsupported at-rule",
            Self::Property => "unsupported property",
            Self::Selector => "unsupported selector",
            Self::Function => "unsupported function",
            Self::Unit => "unsupported unit",
            Self::PseudoElement => "unsupported pseudo-element",
            // Not "unsupported": ignoring another vendor's property is the
            // correct and complete handling of it, and calling that a fault
            // would make a portable stylesheet look broken.
            Self::VendorProperty => "ignored vendor property",
        }
    }
}
