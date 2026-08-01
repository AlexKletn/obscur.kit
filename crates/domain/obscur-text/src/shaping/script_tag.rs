#![doc = include_str!("script_tag.doc/_module.md")]

#[doc = include_str!("script_tag.doc/ScriptTag.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptTag([u8; 4]);

impl ScriptTag {
    #[doc = include_str!("script_tag.doc/LATIN.md")]
    pub const LATIN: ScriptTag = ScriptTag(*b"Latn");
    #[doc = include_str!("script_tag.doc/CYRILLIC.md")]
    pub const CYRILLIC: ScriptTag = ScriptTag(*b"Cyrl");
    #[doc = include_str!("script_tag.doc/HAN.md")]
    pub const HAN: ScriptTag = ScriptTag(*b"Hani");
    #[doc = include_str!("script_tag.doc/ARABIC.md")]
    pub const ARABIC: ScriptTag = ScriptTag(*b"Arab");
    #[doc = include_str!("script_tag.doc/HEBREW.md")]
    pub const HEBREW: ScriptTag = ScriptTag(*b"Hebr");
    #[doc = include_str!("script_tag.doc/GREEK.md")]
    pub const GREEK: ScriptTag = ScriptTag(*b"Grek");
    #[doc = include_str!("script_tag.doc/HIRAGANA.md")]
    pub const HIRAGANA: ScriptTag = ScriptTag(*b"Hira");
    #[doc = include_str!("script_tag.doc/KATAKANA.md")]
    pub const KATAKANA: ScriptTag = ScriptTag(*b"Kana");
    #[doc = include_str!("script_tag.doc/COMMON.md")]
    pub const COMMON: ScriptTag = ScriptTag(*b"Zyyy");
    #[doc = include_str!("script_tag.doc/UNKNOWN.md")]
    pub const UNKNOWN: ScriptTag = ScriptTag(*b"Zzzz");

    #[doc = include_str!("script_tag.doc/from_bytes.md")]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }

    #[doc = include_str!("script_tag.doc/as_bytes.md")]
    #[must_use]
    pub const fn as_bytes(&self) -> [u8; 4] {
        self.0
    }

    #[doc = include_str!("script_tag.doc/as_str.md")]
    #[must_use]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.0).unwrap_or("Zzzz")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_str_round_trip() {
        let tag = ScriptTag::from_bytes(*b"Latn");
        assert_eq!(tag.as_bytes(), *b"Latn");
        assert_eq!(tag.as_str(), "Latn");
        assert_eq!(ScriptTag::from_bytes(tag.as_bytes()), tag);
    }

    #[test]
    fn named_consts_have_expected_codes() {
        assert_eq!(ScriptTag::LATIN.as_str(), "Latn");
        assert_eq!(ScriptTag::CYRILLIC.as_str(), "Cyrl");
        assert_eq!(ScriptTag::HAN.as_str(), "Hani");
        assert_eq!(ScriptTag::ARABIC.as_str(), "Arab");
        assert_eq!(ScriptTag::HEBREW.as_str(), "Hebr");
        assert_eq!(ScriptTag::GREEK.as_str(), "Grek");
        assert_eq!(ScriptTag::HIRAGANA.as_str(), "Hira");
        assert_eq!(ScriptTag::KATAKANA.as_str(), "Kana");
        assert_eq!(ScriptTag::COMMON.as_str(), "Zyyy");
        assert_eq!(ScriptTag::UNKNOWN.as_str(), "Zzzz");
    }

    #[test]
    fn named_consts_are_distinct() {
        let all = [
            ScriptTag::LATIN,
            ScriptTag::CYRILLIC,
            ScriptTag::HAN,
            ScriptTag::ARABIC,
            ScriptTag::HEBREW,
            ScriptTag::GREEK,
            ScriptTag::HIRAGANA,
            ScriptTag::KATAKANA,
            ScriptTag::COMMON,
            ScriptTag::UNKNOWN,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b, "{} vs {}", a.as_str(), b.as_str());
            }
        }
    }
}
