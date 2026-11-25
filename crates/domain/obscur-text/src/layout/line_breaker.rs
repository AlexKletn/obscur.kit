use super::{LineBreak, WhiteSpace};

#[doc = include_str!("../../docs/api/layout/line_break.doc/LineBreaker.md")]
pub trait LineBreaker {
    /// Returns the line-break opportunities in `text` under a `white_space` mode,
    /// in ascending offset order (spec §5.10, UAX #14).
    fn opportunities(&self, text: &str, white_space: WhiteSpace) -> Vec<LineBreak>;
}

#[cfg(test)]
mod tests {
    use super::super::DefaultLineBreaker;
    use super::*;

    #[test]
    fn default_breaker_is_object_safe() {
        let breaker: &dyn LineBreaker = &DefaultLineBreaker;
        assert!(
            breaker
                .opportunities("hello", WhiteSpace::Normal)
                .is_empty()
        );
    }
}
