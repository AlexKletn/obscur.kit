#![doc = include_str!("custom_value_parser.doc/_module.md")]

use crate::property::{PropertyId, PropertyValue};

#[doc = include_str!("custom_value_parser.doc/CustomValueParser.md")]
pub trait CustomValueParser {
    #[doc = include_str!("custom_value_parser.doc/parse.md")]
    fn parse(&self, property: PropertyId, tokens: &str) -> Option<PropertyValue>;
}

// The installer already has a function — the CSS frontend's lowering entry
// point — and would otherwise have to wrap it in a newtype that exists only to
// carry this one method. A closure is the port.
impl<F> CustomValueParser for F
where
    F: Fn(PropertyId, &str) -> Option<PropertyValue>,
{
    fn parse(&self, property: PropertyId, tokens: &str) -> Option<PropertyValue> {
        self(property, tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_closure_is_a_parser() {
        let parser = |property: PropertyId, tokens: &str| {
            (property == PropertyId::FontFamily).then(|| PropertyValue::font_families([tokens]))
        };
        assert_eq!(
            parser.parse(PropertyId::FontFamily, "Inter"),
            Some(PropertyValue::font_families(["Inter"]))
        );
        assert_eq!(parser.parse(PropertyId::Color, "Inter"), None);
    }

    #[test]
    fn a_boxed_closure_is_a_parser_too() {
        // The shape the runtime stores: the port is object-safe, so the
        // installed grammar can be owned without naming its type.
        let parser: Box<dyn CustomValueParser> =
            Box::new(|_: PropertyId, _: &str| Some(PropertyValue::Number(1.0)));
        assert_eq!(
            parser.parse(PropertyId::Opacity, "anything"),
            Some(PropertyValue::Number(1.0))
        );
    }
}
