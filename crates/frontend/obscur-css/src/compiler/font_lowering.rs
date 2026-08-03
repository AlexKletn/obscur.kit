#![doc = include_str!("font_lowering.doc/_module.md")]

use super::property_compiler::narrow_f32;
use super::value_lowering::css_wide_keyword;
use crate::parser::ParsedValue;
use obscur_style::property::{PropertyValue, layout_keyword};

#[doc = include_str!("font_lowering.doc/lower_font_family.md")]
pub(super) fn lower_font_family(values: &[ParsedValue]) -> Option<PropertyValue> {
    let mut families: Vec<Box<str>> = Vec::new();
    for group in values.split(|value| matches!(value, ParsedValue::Comma)) {
        if let Some(name) = font_family_name(group) {
            families.push(name);
        }
    }
    if families.is_empty() {
        return None;
    }
    Some(PropertyValue::FontFamilies(families.into_boxed_slice()))
}

#[doc = include_str!("font_lowering.doc/font_family_name.md")]
fn font_family_name(group: &[ParsedValue]) -> Option<Box<str>> {
    let mut idents: Vec<&str> = Vec::new();
    for value in group {
        match value {
            // A quoted family name or a bare ident both contribute their text.
            ParsedValue::String(text) | ParsedValue::Keyword(text) => idents.push(text),
            _ => {}
        }
    }
    if idents.is_empty() {
        None
    } else {
        Some(idents.join(" ").into_boxed_str())
    }
}

#[doc = include_str!("font_lowering.doc/lower_font_weight.md")]
pub(super) fn lower_font_weight(values: &[ParsedValue]) -> Option<PropertyValue> {
    match values.first()? {
        ParsedValue::Number(n) => Some(PropertyValue::Number(narrow_f32(*n))),
        ParsedValue::Keyword(keyword) => {
            if let Some(wide) = css_wide_keyword(keyword) {
                return Some(wide);
            }
            if let Some(symbol) = layout_keyword::font_weight_relative_from_name(keyword) {
                return Some(PropertyValue::Keyword(symbol));
            }
            font_weight_keyword(keyword).map(PropertyValue::Number)
        }
        _ => None,
    }
}

#[doc = include_str!("font_lowering.doc/font_weight_keyword.md")]
fn font_weight_keyword(keyword: &str) -> Option<f32> {
    match keyword.to_ascii_lowercase().as_str() {
        "normal" => Some(400.0),
        "bold" => Some(700.0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::property_compiler::tests::{standard, value_of};

    use obscur_style::property::{PropertyId, PropertyValue, layout_keyword};

    /// Extracts the ordered family names from a lowered `font-family` value.
    fn families(text: &str) -> Vec<String> {
        match value_of(text) {
            PropertyValue::FontFamilies(names) => {
                names.iter().map(std::string::ToString::to_string).collect()
            }
            other => panic!("expected font families, got {other:?}"),
        }
    }

    #[test]
    fn lowers_font_family_list_in_order() {
        // Quoted string, bare ident, and generic keyword, in author order.
        assert_eq!(
            families(r#"font-family: "Foo", Arial, sans-serif"#),
            vec![
                "Foo".to_string(),
                "Arial".to_string(),
                "sans-serif".to_string()
            ]
        );
    }

    #[test]
    fn lowers_multi_ident_family_joined_by_spaces() {
        // Unquoted multi-word family names join their idents with single spaces.
        assert_eq!(
            families("font-family: Helvetica Neue, monospace"),
            vec!["Helvetica Neue".to_string(), "monospace".to_string()]
        );
    }

    #[test]
    fn font_family_carries_its_property_id() {
        assert_eq!(
            standard("font-family: Arial").0.property(),
            PropertyId::FontFamily
        );
    }

    #[test]
    fn font_family_honours_css_wide_keywords() {
        assert_eq!(value_of("font-family: inherit"), PropertyValue::Inherit);
        assert_eq!(value_of("font-family: initial"), PropertyValue::Initial);
    }

    #[test]
    fn font_weight_lowers_numbers_and_keywords() {
        assert_eq!(value_of("font-weight: 300"), PropertyValue::Number(300.0));
        assert_eq!(value_of("font-weight: 600"), PropertyValue::Number(600.0));
        assert_eq!(value_of("font-weight: bold"), PropertyValue::Number(700.0));
        assert_eq!(
            value_of("font-weight: normal"),
            PropertyValue::Number(400.0)
        );
        assert_eq!(
            standard("font-weight: 500").0.property(),
            PropertyId::FontWeight
        );
    }

    #[test]
    fn font_weight_relative_keywords_lower_to_reserved_handles() {
        // `bolder` / `lighter` are carried as reserved handles so the computed-
        // value step can resolve them relative to the parent (§4.14), rather than
        // being flattened to fixed numbers here.
        assert_eq!(
            value_of("font-weight: bolder"),
            PropertyValue::Keyword(layout_keyword::FONT_WEIGHT_BOLDER)
        );
        assert_eq!(
            value_of("font-weight: lighter"),
            PropertyValue::Keyword(layout_keyword::FONT_WEIGHT_LIGHTER)
        );
    }
}
