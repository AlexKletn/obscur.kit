#![doc = include_str!("important_flag.doc/_module.md")]

#[doc = include_str!("important_flag.doc/ImportantFlag.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ImportantFlag {
    #[doc = include_str!("important_flag.doc/Normal.md")]
    Normal,
    #[doc = include_str!("important_flag.doc/Important.md")]
    Important,
}

impl ImportantFlag {
    #[doc = include_str!("important_flag.doc/is_important.md")]
    #[must_use]
    pub const fn is_important(self) -> bool {
        matches!(self, Self::Important)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn important_outranks_normal() {
        assert!(ImportantFlag::Important > ImportantFlag::Normal);
    }

    #[test]
    fn is_important_predicate() {
        assert!(ImportantFlag::Important.is_important());
        assert!(!ImportantFlag::Normal.is_important());
    }
}
