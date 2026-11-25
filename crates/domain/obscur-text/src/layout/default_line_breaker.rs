use super::{LineBreak, LineBreaker, WhiteSpace, is_cjk_ideograph::is_cjk_ideograph};

#[doc = include_str!("../../docs/api/layout/line_break.doc/DefaultLineBreaker.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DefaultLineBreaker;

impl LineBreaker for DefaultLineBreaker {
    fn opportunities(&self, text: &str, white_space: WhiteSpace) -> Vec<LineBreak> {
        let mut breaks = LineBreak::opportunities(text, white_space);
        // A break opportunity sits between two adjacent CJK ideographs, but only
        // where soft wrapping is permitted (§3). This is additive: for text with
        // no ideograph pair the loop pushes nothing and `breaks` is returned
        // untouched (byte-identical to `LineBreak::opportunities`).
        if white_space.wraps() {
            let before = breaks.len();
            let mut prev: Option<char> = None;
            for (index, ch) in text.char_indices() {
                if let Some(prev_ch) = prev
                    && is_cjk_ideograph(prev_ch)
                    && is_cjk_ideograph(ch)
                {
                    breaks.push(LineBreak::soft(index));
                }
                prev = Some(ch);
            }
            if breaks.len() != before {
                breaks.sort_unstable();
                breaks.dedup();
            }
        }
        breaks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_breaker_reproduces_opportunities_for_non_cjk_text() {
        // Requirement (d): the port default is byte-identical to
        // `LineBreak::opportunities` for every whitespace mode on Latin text.
        for text in ["hello", "a b\nc", "aa bb cc", "é  x", "ab"] {
            for mode in [
                WhiteSpace::Normal,
                WhiteSpace::Nowrap,
                WhiteSpace::Pre,
                WhiteSpace::PreWrap,
                WhiteSpace::PreLine,
            ] {
                assert_eq!(
                    DefaultLineBreaker.opportunities(text, mode),
                    LineBreak::opportunities(text, mode),
                    "{text:?} / {mode:?}"
                );
            }
        }
    }

    #[test]
    fn default_breaker_adds_soft_break_between_adjacent_cjk_ideographs() {
        // "中文字" — three CJK ideographs (3 bytes each): soft breaks before the
        // 2nd (byte 3) and 3rd (byte 6) so unspaced CJK wraps out of the box.
        let breaks = DefaultLineBreaker.opportunities("中文字", WhiteSpace::Normal);
        assert_eq!(breaks, vec![LineBreak::soft(3), LineBreak::soft(6)]);
        // A non-wrapping mode (`nowrap`) yields no CJK soft breaks.
        assert!(
            DefaultLineBreaker
                .opportunities("中文字", WhiteSpace::Nowrap)
                .is_empty()
        );
    }
}
