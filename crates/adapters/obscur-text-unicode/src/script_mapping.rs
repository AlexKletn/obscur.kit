#![doc = include_str!("script_mapping.doc/_module.md")]

use obscur_text::shaping::ScriptTag;
use unicode_script::{Script, UnicodeScript};

#[doc = include_str!("script_mapping.doc/script_tag_for.md")]
pub(crate) fn script_tag_for(ch: char) -> Option<ScriptTag> {
    match ch.script() {
        Script::Latin => Some(ScriptTag::LATIN),
        Script::Cyrillic => Some(ScriptTag::CYRILLIC),
        Script::Han => Some(ScriptTag::HAN),
        Script::Arabic => Some(ScriptTag::ARABIC),
        Script::Hebrew => Some(ScriptTag::HEBREW),
        Script::Greek => Some(ScriptTag::GREEK),
        Script::Hiragana => Some(ScriptTag::HIRAGANA),
        Script::Katakana => Some(ScriptTag::KATAKANA),
        // Script-neutral: attach to the surrounding run (never break here).
        Script::Common | Script::Inherited => None,
        // Any other assigned script the domain does not name explicitly.
        _ => Some(ScriptTag::UNKNOWN),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concrete_scripts_map_to_named_tags() {
        assert_eq!(script_tag_for('a'), Some(ScriptTag::LATIN));
        assert_eq!(script_tag_for('Я'), Some(ScriptTag::CYRILLIC));
        assert_eq!(script_tag_for('中'), Some(ScriptTag::HAN));
        assert_eq!(script_tag_for('ا'), Some(ScriptTag::ARABIC));
        assert_eq!(script_tag_for('ש'), Some(ScriptTag::HEBREW));
        assert_eq!(script_tag_for('α'), Some(ScriptTag::GREEK));
        assert_eq!(script_tag_for('あ'), Some(ScriptTag::HIRAGANA));
        assert_eq!(script_tag_for('カ'), Some(ScriptTag::KATAKANA));
    }

    #[test]
    fn neutral_scripts_yield_none() {
        assert_eq!(script_tag_for(' '), None); // space (Common)
        assert_eq!(script_tag_for('!'), None); // punctuation (Common)
        assert_eq!(script_tag_for('1'), None); // digit (Common)
        assert_eq!(script_tag_for('\u{0301}'), None); // combining acute (Inherited)
    }

    #[test]
    fn unnamed_script_maps_to_unknown() {
        // Devanagari is a concrete script the domain does not name explicitly.
        assert_eq!(script_tag_for('अ'), Some(ScriptTag::UNKNOWN));
    }
}
