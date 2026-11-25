#[doc = include_str!("../../docs/api/layout/line_break.doc/is_cjk_ideograph.md")]
#[must_use]
pub(crate) fn is_cjk_ideograph(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF      // CJK Unified Ideographs Extension A
        | 0x4E00..=0x9FFF    // CJK Unified Ideographs
        | 0xF900..=0xFAFF    // CJK Compatibility Ideographs
        | 0x2_0000..=0x2_A6DF // Extension B
        | 0x2_A700..=0x2_EBEF // Extensions C–F
        | 0x2_F800..=0x2_FA1F // CJK Compatibility Ideographs Supplement
        | 0x3_0000..=0x3_134F // Extension G
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjk_ideograph_range() {
        assert!(is_cjk_ideograph('中'));
        assert!(is_cjk_ideograph('字'));
        assert!(!is_cjk_ideograph('a'));
        assert!(!is_cjk_ideograph('Я'));
        // Hiragana is deliberately excluded from the simplified default set.
        assert!(!is_cjk_ideograph('あ'));
    }
}
