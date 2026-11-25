#[doc = include_str!("../../docs/api/selector/pseudo_class_selector.doc/AnB.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnB {
    /// The step (`a` in `An+B`).
    pub a: i32,
    /// The offset (`b` in `An+B`).
    pub b: i32,
}

impl AnB {
    /// Creates an `An+B` value from its step `a` and offset `b`.
    #[must_use]
    pub const fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }

    #[doc = include_str!("../../docs/api/selector/pseudo_class_selector.doc/matches_index.md")]
    #[must_use]
    pub const fn matches_index(&self, one_based_index: i32) -> bool {
        let diff = one_based_index as i64 - self.b as i64;
        let step = self.a as i64;
        if step == 0 {
            return diff == 0;
        }
        // `n = diff / a` must be a non-negative integer.
        diff % step == 0 && diff / step >= 0
    }
}

#[cfg(test)]
mod tests {
    use super::AnB;

    #[test]
    fn anb_two_n_plus_one_matches_odd() {
        let anb = AnB::new(2, 1);
        assert!(anb.matches_index(1));
        assert!(anb.matches_index(3));
        assert!(anb.matches_index(5));
        assert!(!anb.matches_index(2));
        assert!(!anb.matches_index(4));
    }

    #[test]
    fn anb_constant_matches_only_that_index() {
        let anb = AnB::new(0, 3);
        assert!(anb.matches_index(3));
        assert!(!anb.matches_index(1));
        assert!(!anb.matches_index(6));
    }

    #[test]
    fn anb_zero_matches_nothing_for_one_based() {
        let anb = AnB::new(0, 0);
        assert!(!anb.matches_index(1));
        assert!(!anb.matches_index(2));
    }

    #[test]
    fn anb_negative_step_matches_leading_range() {
        let anb = AnB::new(-1, 3);
        assert!(anb.matches_index(1));
        assert!(anb.matches_index(2));
        assert!(anb.matches_index(3));
        assert!(!anb.matches_index(4));
    }

    #[test]
    fn anb_even_matches_two_n() {
        let anb = AnB::new(2, 0);
        assert!(anb.matches_index(2));
        assert!(anb.matches_index(4));
        assert!(!anb.matches_index(1));
        assert!(!anb.matches_index(3));
    }

    #[test]
    fn accepts_extreme_offsets_without_overflow() {
        assert!(AnB::new(1, i32::MIN).matches_index(i32::MAX));
        assert!(!AnB::new(-1, i32::MIN).matches_index(1));
        assert!(AnB::new(-1, i32::MAX).matches_index(1));
    }
}
