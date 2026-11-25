//! An ISO-15924 four-letter script identifier (spec §5.10).

#[doc = include_str!("../../docs/api/shaping/script_tag.doc/ScriptTag.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptTag([u8; 4]);

impl ScriptTag {
    /// Latin script (`Latn`).
    pub const LATIN: ScriptTag = ScriptTag(*b"Latn");
    /// Cyrillic script (`Cyrl`).
    pub const CYRILLIC: ScriptTag = ScriptTag(*b"Cyrl");
    /// Han (CJK ideographic) script (`Hani`).
    pub const HAN: ScriptTag = ScriptTag(*b"Hani");
    /// Arabic script (`Arab`).
    pub const ARABIC: ScriptTag = ScriptTag(*b"Arab");
    /// Hebrew script (`Hebr`).
    pub const HEBREW: ScriptTag = ScriptTag(*b"Hebr");
    /// Greek script (`Grek`).
    pub const GREEK: ScriptTag = ScriptTag(*b"Grek");
    /// Hiragana script (`Hira`).
    pub const HIRAGANA: ScriptTag = ScriptTag(*b"Hira");
    /// Katakana script (`Kana`).
    pub const KATAKANA: ScriptTag = ScriptTag(*b"Kana");
    /// The common / script-neutral pseudo-script (`Zyyy`).
    pub const COMMON: ScriptTag = ScriptTag(*b"Zyyy");
    /// The unknown / unassigned pseudo-script (`Zzzz`).
    pub const UNKNOWN: ScriptTag = ScriptTag(*b"Zzzz");

    /// Creates a tag from its four raw ISO-15924 bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }

    /// Returns the tag's four raw bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> [u8; 4] {
        self.0
    }

    #[doc = include_str!("../../docs/api/shaping/script_tag.doc/as_str.md")]
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
