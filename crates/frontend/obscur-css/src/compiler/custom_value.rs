#![doc = include_str!("../../docs/api/compiler/custom_value.doc/_module.md")]

use std::cell::RefCell;
use std::fmt::Write as _;

use obscur_identity::FastMap;
use obscur_style::computed::CustomValueParser;
use obscur_style::matching::StyleRangeValue;
use obscur_style::property::{Length, PropertyId, PropertyValue};

use crate::parser::{ParsedValue, ValueParser};
use crate::source::CssSource;

use super::CssSymbolInterner;
use super::condition_lowering::lower_style_literal;
use super::property_compiler::lower_for_property;
use super::shorthand_expansion::{container_values, flex_flow_values, pair_values};
use super::value_lowering::dimension_length;

#[doc = include_str!("../../docs/api/compiler/custom_value.doc/custom_value_text.md")]
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

#[doc = include_str!("../../docs/api/compiler/custom_value.doc/lower_custom_value.md")]
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

#[doc = include_str!("../../docs/api/compiler/custom_value.doc/LoweredByText.md")]
type LoweredByText = FastMap<Box<str>, Option<PropertyValue>>;

type LoweredContainers = FastMap<Box<str>, Option<(PropertyValue, PropertyValue)>>;

#[doc = include_str!("../../docs/api/compiler/custom_value.doc/CustomValueCache.md")]
#[derive(Debug, Default)]
pub struct CustomValueCache {
    #[doc = include_str!("../../docs/api/compiler/custom_value.doc/lowered.md")]
    lowered: RefCell<FastMap<PropertyId, LoweredByText>>,
    style_properties: RefCell<FastMap<PropertyId, LoweredByText>>,
    containers: RefCell<LoweredContainers>,
    gaps: RefCell<FastMap<Box<str>, Option<[PropertyValue; 2]>>>,
    flex_flows: RefCell<FastMap<Box<str>, Option<[PropertyValue; 2]>>>,
    ranges: RefCell<FastMap<Box<str>, Option<StyleRangeValue>>>,
}

impl CustomValueCache {
    /// An empty cache, which answers exactly as [`lower_custom_value`] does.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl CustomValueParser for CustomValueCache {
    fn parse(&self, property: PropertyId, tokens: &str) -> Option<PropertyValue> {
        // Borrowed and released before the parse below: `lower_custom_value`
        // does not reach back into the cache, but holding a `Ref` across a call
        // that could is how a `RefCell` comes to panic later.
        if let Some(hit) = self
            .lowered
            .borrow()
            .get(&property)
            .and_then(|by_text| by_text.get(tokens))
        {
            return hit.clone();
        }
        let lowered = lower_custom_value(property, tokens);
        // A miss is remembered too. `None` is the answer for every `var()` whose
        // text no grammar of this property accepts, and a stylesheet that has
        // one asks about it on every node exactly as often as a hit.
        self.lowered
            .borrow_mut()
            .entry(property)
            .or_default()
            .insert(tokens.into(), lowered.clone());
        lowered
    }

    fn parse_style_property(&self, property: PropertyId, tokens: &str) -> Option<PropertyValue> {
        if let Some(hit) = self
            .style_properties
            .borrow()
            .get(&property)
            .and_then(|by_text| by_text.get(tokens))
        {
            return hit.clone();
        }
        let lowered = ValueParser::new()
            .parse(&CssSource::anonymous(tokens))
            .ok()
            .and_then(|values| lower_style_literal(property, &values));
        self.style_properties
            .borrow_mut()
            .entry(property)
            .or_default()
            .insert(tokens.into(), lowered.clone());
        lowered
    }

    fn parse_style_range(&self, tokens: &str) -> Option<StyleRangeValue> {
        if let Some(hit) = self.ranges.borrow().get(tokens) {
            return hit.clone();
        }
        let value = ValueParser::new()
            .parse(&CssSource::anonymous(tokens))
            .ok()
            .and_then(|values| lower_style_range(&values));
        self.ranges
            .borrow_mut()
            .insert(tokens.into(), value.clone());
        value
    }

    fn parse_gap(&self, tokens: &str) -> Option<[PropertyValue; 2]> {
        if let Some(hit) = self.gaps.borrow().get(tokens) {
            return hit.clone();
        }
        let lowered = ValueParser::new()
            .parse(&CssSource::anonymous(tokens))
            .ok()
            .and_then(|values| {
                pair_values(
                    PropertyId::RowGap,
                    PropertyId::ColumnGap,
                    &values,
                    &mut CssSymbolInterner::new(),
                )
            });
        self.gaps
            .borrow_mut()
            .insert(tokens.into(), lowered.clone());
        lowered
    }

    fn parse_flex_flow(&self, tokens: &str) -> Option<[PropertyValue; 2]> {
        if let Some(hit) = self.flex_flows.borrow().get(tokens) {
            return hit.clone();
        }
        let lowered = ValueParser::new()
            .parse(&CssSource::anonymous(tokens))
            .ok()
            .and_then(|values| flex_flow_values(&values));
        self.flex_flows
            .borrow_mut()
            .insert(tokens.into(), lowered.clone());
        lowered
    }

    fn parse_container(&self, tokens: &str) -> Option<(PropertyValue, PropertyValue)> {
        if let Some(hit) = self.containers.borrow().get(tokens) {
            return hit.clone();
        }
        let lowered = ValueParser::new()
            .parse(&CssSource::anonymous(tokens))
            .ok()
            .and_then(|values| container_values(&values, &mut CssSymbolInterner::new()));
        self.containers
            .borrow_mut()
            .insert(tokens.into(), lowered.clone());
        lowered
    }
}

pub(super) fn lower_style_range(values: &[ParsedValue]) -> Option<StyleRangeValue> {
    use super::property_compiler::narrow_f32;
    match values {
        [ParsedValue::Number(value)] => Some(StyleRangeValue::Number(narrow_f32(*value))),
        [ParsedValue::Percentage(value)] => Some(StyleRangeValue::Percent(narrow_f32(*value))),
        [ParsedValue::Dimension { value, unit }] => {
            let length = dimension_length(*value, unit)?;
            Some(match length {
                Length::Px(px) => StyleRangeValue::Pixels(px.0),
                length => StyleRangeValue::Relative(length),
            })
        }
        [ParsedValue::Function { name, args }] => {
            super::calc_lowering::lower_range_math(name, args)
        }
        _ => None,
    }
}

#[doc = include_str!("../../docs/api/compiler/custom_value.doc/write_component.md")]
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

    #[test]
    fn the_cache_answers_what_the_bare_lowering_answers() {
        // The whole contract. Anything the cache says that the function does
        // not is a stylesheet reading differently depending on whether it was
        // read before, which is worse than the cost it saves.
        let cache = CustomValueCache::new();
        for (property, text) in [
            (PropertyId::FontFamily, "Inter, sans-serif"),
            (PropertyId::Color, "#ff0000"),
            (PropertyId::Width, "12px"),
            (PropertyId::BoxShadow, "0 2px 4px #000000"),
            (PropertyId::OverflowX, "hidden"),
            // The same text on a property whose grammar rejects it, and a word
            // no grammar has: both are answers worth remembering too.
            (PropertyId::FontFamily, "12px"),
            (PropertyId::OverflowX, "sideways"),
        ] {
            assert_eq!(
                cache.parse(property, text),
                lower_custom_value(property, text),
                "{property:?} {text}"
            );
        }
    }

    #[test]
    fn asking_twice_reads_the_answer_back_rather_than_parsing_again() {
        let cache = CustomValueCache::new();
        let first = cache.parse(PropertyId::FontFamily, "Inter, sans-serif");
        let second = cache.parse(PropertyId::FontFamily, "Inter, sans-serif");
        assert_eq!(first, second);
        assert_eq!(
            cache.lowered.borrow()[&PropertyId::FontFamily].len(),
            1,
            "one entry, not one per ask"
        );
    }

    #[test]
    fn one_text_is_remembered_per_property_it_was_asked_against() {
        // `12px` is a width and is no family, and the cache may not let the
        // first question it was asked answer the second.
        let cache = CustomValueCache::new();
        assert_eq!(
            cache.parse(PropertyId::Width, "12px"),
            Some(PropertyValue::Length(Length::px(12.0)))
        );
        assert_eq!(cache.parse(PropertyId::FontFamily, "12px"), None);
        assert_eq!(
            cache.parse(PropertyId::Width, "12px"),
            Some(PropertyValue::Length(Length::px(12.0))),
            "and the first answer survived the second question"
        );
    }
}
