#![doc = include_str!("custom_value.doc/_module.md")]

use std::fmt::Write as _;

use obscur_style::property::{PropertyId, PropertyValue};

use crate::parser::{ParsedValue, ValueParser};
use crate::source::CssSource;

use super::property_compiler::lower_for_property;
use super::stylesheet_compiler::CssSymbolInterner;

#[doc = include_str!("custom_value.doc/custom_value_text.md")]
pub(super) fn custom_value_text(values: &[ParsedValue]) -> Box<str> {
    let mut out = String::new();
    for value in values {
        if !out.is_empty() && !matches!(value, ParsedValue::Comma) {
            out.push(' ');
        }
        write_component(&mut out, value);
    }
    out.into_boxed_str()
}

#[doc = include_str!("custom_value.doc/lower_custom_value.md")]
#[must_use]
pub fn lower_custom_value(property: PropertyId, tokens: &str) -> Option<PropertyValue> {
    let values = ValueParser::new()
        .parse(&CssSource::anonymous(tokens))
        .ok()?;
    // The interner is a THROWAWAY: it is dropped when this function returns, so
    // any symbol minted into it is a name nothing can ever read back. That is
    // survivable for the properties whose keywords are open — nothing reads
    // those symbols as values — and it is exactly why the closed-set properties
    // may not mint one at all. That rule lives in `lower_for_property`, where
    // the literal spelling of the same declaration asks it too; this path used
    // to keep a second copy of it, and the two drifted the moment only one was
    // fixed.
    let mut interner = CssSymbolInterner::new();
    lower_for_property(property, &values, &mut interner)
}

#[doc = include_str!("custom_value.doc/write_component.md")]
fn write_component(out: &mut String, value: &ParsedValue) {
    match value {
        ParsedValue::Keyword(word) => out.push_str(word),
        ParsedValue::Number(n) => {
            let _ = write!(out, "{n}");
        }
        ParsedValue::Percentage(p) => {
            let _ = write!(out, "{p}%");
        }
        ParsedValue::Dimension { value, unit } => {
            let _ = write!(out, "{value}{unit}");
        }
        // Re-quoted, because the tokenizer stripped the quotes and a bare
        // `Inter Tight` would come back as two components rather than one
        // family whose name has a space in it.
        ParsedValue::String(text) => {
            out.push('"');
            for ch in text.chars() {
                if ch == '"' || ch == '\\' {
                    out.push('\\');
                }
                out.push(ch);
            }
            out.push('"');
        }
        ParsedValue::Hash(hex) => {
            out.push('#');
            out.push_str(hex);
        }
        ParsedValue::Function { name, args } => {
            out.push_str(name);
            out.push('(');
            out.push_str(&custom_value_text(args));
            out.push(')');
        }
        ParsedValue::Comma => out.push(','),
        ParsedValue::Delimiter(ch) => out.push(*ch),
        ParsedValue::LineNames(names) => {
            out.push('[');
            out.push_str(&names.join(" "));
            out.push(']');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use obscur_style::property::{Length, StyleColor, layout_keyword};

    fn parsed(text: &str) -> Vec<ParsedValue> {
        ValueParser::new()
            .parse(&CssSource::anonymous(text))
            .expect("the test value parses")
    }

    fn round_trip(text: &str) -> Vec<ParsedValue> {
        parsed(&custom_value_text(&parsed(text)))
    }

    #[test]
    fn writing_a_value_out_and_reading_it_back_yields_the_same_components() {
        // The only contract this writer has. Each of these is a shape a custom
        // property is actually written with, and each used to lose everything
        // past its first component.
        for text in [
            "Inter, sans-serif",
            "0 2px 4px #000000",
            "\"Inter Tight\", Georgia, serif",
            "calc(100% - 20px)",
            "1px solid rgba(0, 0, 0, 0.5)",
            "linear-gradient(90deg, #ff0000, #0000ff)",
            "translate(4px, -8px) scale(1.5)",
            "0 0 0 1px #fff, 0 2px 4px #000",
        ] {
            assert_eq!(round_trip(text), parsed(text), "{text}");
        }
    }

    #[test]
    fn a_quoted_family_stays_one_component() {
        // Unquoted it would tokenize as two keywords and become two families.
        assert_eq!(
            custom_value_text(&parsed("\"Inter Tight\"")).as_ref(),
            "\"Inter Tight\""
        );
    }

    #[test]
    fn text_is_lowered_against_the_property_it_lands_on() {
        assert_eq!(
            lower_custom_value(PropertyId::FontFamily, "Inter, sans-serif"),
            Some(PropertyValue::font_families(["Inter", "sans-serif"])),
            "both families, in order"
        );
        assert_eq!(
            lower_custom_value(PropertyId::Color, "#ff0000"),
            Some(PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)))
        );
        assert_eq!(
            lower_custom_value(PropertyId::Width, "12px"),
            Some(PropertyValue::Length(Length::px(12.0)))
        );
        let shadow = lower_custom_value(PropertyId::BoxShadow, "0 2px 4px #000000");
        assert!(
            matches!(shadow, Some(PropertyValue::BoxShadow(ref list)) if list.len() == 1),
            "one shadow with all four components, got {shadow:?}"
        );
    }

    #[test]
    fn the_same_text_means_different_things_to_different_properties() {
        // The reason this parse cannot happen at declaration time: `12px` is a
        // width and is no font family, and which one it is depends on a
        // property the declaration does not name.
        assert_eq!(
            lower_custom_value(PropertyId::Width, "12px"),
            Some(PropertyValue::Length(Length::px(12.0)))
        );
        assert_eq!(lower_custom_value(PropertyId::FontFamily, "12px"), None);
    }

    #[test]
    fn a_word_a_closed_keyword_property_does_not_have_is_no_value() {
        // `overflow-x` has five words and `sideways` is none of them. Interning
        // it minted a symbol into the interner this call drops, so no consumer
        // could read the name back — and `OverflowAxes` reads every symbol that
        // is not `visible` or `clip` as clipping, so a typo in a theme variable
        // clipped the box.
        assert_eq!(lower_custom_value(PropertyId::OverflowX, "sideways"), None);
        assert_eq!(
            lower_custom_value(PropertyId::OverflowX, "hidden"),
            Some(PropertyValue::Keyword(layout_keyword::OVERFLOW_HIDDEN)),
            "a word it does have still lowers to the reserved symbol"
        );
    }

    #[test]
    fn text_that_matches_no_value_of_the_property_is_no_value() {
        assert_eq!(
            lower_custom_value(PropertyId::BoxShadow, "not a shadow at all"),
            None
        );
        assert_eq!(lower_custom_value(PropertyId::Transform, "sideways"), None);
    }
}
