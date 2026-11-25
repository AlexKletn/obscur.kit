//! CSS selector specificity: the ordered `(a, b, c)` triple.

#[doc = include_str!("../../docs/api/selector/selector_specificity.doc/SelectorSpecificity.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SelectorSpecificity {
    /// Number of ID selectors (most significant component).
    a: u16,
    /// Number of class / attribute / pseudo-class selectors.
    b: u16,
    /// Number of type / pseudo-element selectors (least significant component).
    c: u16,
}

impl SelectorSpecificity {
    #[doc = include_str!("../../docs/api/selector/selector_specificity.doc/ZERO.md")]
    pub const ZERO: Self = Self { a: 0, b: 0, c: 0 };

    /// Creates a specificity from its three components `(a, b, c)`.
    #[must_use]
    pub const fn new(a: u16, b: u16, c: u16) -> Self {
        Self { a, b, c }
    }

    /// Returns the `a` component: the number of ID selectors.
    #[must_use]
    pub const fn a(self) -> u16 {
        self.a
    }

    /// Returns the `b` component: class / attribute / pseudo-class count.
    #[must_use]
    pub const fn b(self) -> u16 {
        self.b
    }

    /// Returns the `c` component: type / pseudo-element count.
    #[must_use]
    pub const fn c(self) -> u16 {
        self.c
    }

    #[doc = include_str!("../../docs/api/selector/selector_specificity.doc/add.md")]
    #[must_use]
    pub const fn add(self, other: Self) -> Self {
        Self {
            a: self.a.saturating_add(other.a),
            b: self.b.saturating_add(other.b),
            c: self.c.saturating_add(other.c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_identity_for_add() {
        let s = SelectorSpecificity::new(1, 2, 3);
        assert_eq!(s.add(SelectorSpecificity::ZERO), s);
        assert_eq!(SelectorSpecificity::ZERO.add(s), s);
    }

    #[test]
    fn accessors_return_components() {
        let s = SelectorSpecificity::new(4, 5, 6);
        assert_eq!(s.a(), 4);
        assert_eq!(s.b(), 5);
        assert_eq!(s.c(), 6);
    }

    #[test]
    fn add_is_component_wise() {
        let lhs = SelectorSpecificity::new(1, 0, 2);
        let rhs = SelectorSpecificity::new(0, 3, 4);
        assert_eq!(lhs.add(rhs), SelectorSpecificity::new(1, 3, 6));
    }

    #[test]
    fn add_saturates() {
        let max = SelectorSpecificity::new(u16::MAX, u16::MAX, u16::MAX);
        let one = SelectorSpecificity::new(1, 1, 1);
        assert_eq!(max.add(one), max);
    }

    #[test]
    fn one_id_beats_many_classes() {
        let one_id = SelectorSpecificity::new(1, 0, 0);
        let many_classes = SelectorSpecificity::new(0, 10, 0);
        assert!(one_id > many_classes);
    }

    #[test]
    fn ordering_is_lexicographic() {
        // b dominates c.
        assert!(SelectorSpecificity::new(0, 1, 0) > SelectorSpecificity::new(0, 0, 99));
        // a dominates b.
        assert!(SelectorSpecificity::new(1, 0, 0) > SelectorSpecificity::new(0, 99, 99));
        // Equal a, b: compare c.
        assert!(SelectorSpecificity::new(1, 1, 2) > SelectorSpecificity::new(1, 1, 1));
    }

    #[test]
    fn equal_specificities_compare_equal() {
        assert_eq!(
            SelectorSpecificity::new(2, 3, 4),
            SelectorSpecificity::new(2, 3, 4)
        );
    }
}
