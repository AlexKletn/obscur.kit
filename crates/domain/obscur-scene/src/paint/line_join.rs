#![doc = include_str!("line_join.doc/_module.md")]

#[doc = include_str!("line_join.doc/LineJoin.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineJoin {
    #[doc = include_str!("line_join.doc/Miter.md")]
    Miter,
    #[doc = include_str!("line_join.doc/Round.md")]
    Round,
    #[doc = include_str!("line_join.doc/Bevel.md")]
    Bevel,
}

impl Default for LineJoin {
    #[doc = include_str!("line_join.doc/default.md")]
    fn default() -> Self {
        Self::Miter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_miter() {
        assert_eq!(LineJoin::default(), LineJoin::Miter);
    }
}
