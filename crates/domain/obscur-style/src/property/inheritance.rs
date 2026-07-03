//! Inheritance classification for CSS properties.
//!
//! Per the Obscur Kit 2 draft (§4.14), inheritance is decided *per property*:
//! some properties inherit from their parent by default (e.g. `color`), while
//! most do not (e.g. `opacity`, `background-color`). This module models that
//! binary classification as a small, `Copy` enum so the cascade can reason
//! about default inheritance without any string comparison.

/// Whether a property inherits from its parent element by default.
///
/// This captures the property's *default* inheritance behaviour — the CSS-wide
/// keywords `inherit`, `initial` and `unset` (modelled on
/// [`crate::property::PropertyValue`]) can override this per declaration during
/// the cascade, but the default recorded here is what `unset` resolves against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Inheritance {
    /// The property inherits its computed value from the parent by default
    /// (for example `color`).
    Inherited,
    /// The property does not inherit by default; it resets to its initial value
    /// on each element (for example `opacity` or `background-color`).
    NotInherited,
}

impl Inheritance {
    /// Returns `true` when the property inherits from its parent by default.
    #[must_use]
    pub const fn is_inherited(self) -> bool {
        matches!(self, Self::Inherited)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_inherited_reflects_variant() {
        assert!(Inheritance::Inherited.is_inherited());
        assert!(!Inheritance::NotInherited.is_inherited());
    }

    #[test]
    fn is_copy_and_eq() {
        let a = Inheritance::Inherited;
        let b = a;
        assert_eq!(a, b);
    }
}
