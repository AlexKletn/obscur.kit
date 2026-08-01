#![doc = include_str!("inheritance.doc/_module.md")]

#[doc = include_str!("inheritance.doc/Inheritance.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Inheritance {
    #[doc = include_str!("inheritance.doc/Inherited.md")]
    Inherited,
    #[doc = include_str!("inheritance.doc/NotInherited.md")]
    NotInherited,
}

impl Inheritance {
    #[doc = include_str!("inheritance.doc/is_inherited.md")]
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
