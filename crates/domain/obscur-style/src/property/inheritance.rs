#![doc = include_str!("../../docs/api/property/inheritance.doc/_module.md")]

#[doc = include_str!("../../docs/api/property/inheritance.doc/Inheritance.md")]
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
