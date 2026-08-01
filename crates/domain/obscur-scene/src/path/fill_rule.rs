#![doc = include_str!("fill_rule.doc/_module.md")]

#[doc = include_str!("fill_rule.doc/FillRule.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FillRule {
    #[doc = include_str!("fill_rule.doc/NonZero.md")]
    NonZero,
    #[doc = include_str!("fill_rule.doc/EvenOdd.md")]
    EvenOdd,
}

impl Default for FillRule {
    #[doc = include_str!("fill_rule.doc/default.md")]
    fn default() -> Self {
        Self::NonZero
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_non_zero() {
        assert_eq!(FillRule::default(), FillRule::NonZero);
    }
}
