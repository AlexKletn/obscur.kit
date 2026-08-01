#![doc = include_str!("ui_ir_property.doc/_module.md")]

use crate::document::{UiIrExpression, UiIrValue};

#[doc = include_str!("ui_ir_property.doc/UiIrProperty.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct UiIrProperty {
    name: Box<str>,
    value: UiIrExpression,
}

impl UiIrProperty {
    #[doc = include_str!("ui_ir_property.doc/new.md")]
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, value: UiIrExpression) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    #[doc = include_str!("ui_ir_property.doc/literal.md")]
    #[must_use]
    pub fn literal(name: impl Into<Box<str>>, value: UiIrValue) -> Self {
        Self::new(name, UiIrExpression::literal(value))
    }

    #[doc = include_str!("ui_ir_property.doc/text.md")]
    #[must_use]
    pub fn text(name: impl Into<Box<str>>, value: impl Into<Box<str>>) -> Self {
        Self::new(name, UiIrExpression::text(value))
    }

    #[doc = include_str!("ui_ir_property.doc/binding.md")]
    #[must_use]
    pub fn binding(name: impl Into<Box<str>>, path: impl Into<Box<str>>) -> Self {
        Self::new(name, UiIrExpression::binding(path))
    }

    #[doc = include_str!("ui_ir_property.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("ui_ir_property.doc/value.md")]
    #[must_use]
    pub const fn value(&self) -> &UiIrExpression {
        &self.value
    }

    #[doc = include_str!("ui_ir_property.doc/is_dynamic.md")]
    #[must_use]
    pub const fn is_dynamic(&self) -> bool {
        !self.value.is_literal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_holds_name_and_value() {
        let prop = UiIrProperty::new("count", UiIrExpression::int(3));
        assert_eq!(prop.name(), "count");
        assert_eq!(prop.value(), &UiIrExpression::int(3));
    }

    #[test]
    fn literal_and_text_constructors() {
        let lit = UiIrProperty::literal("enabled", UiIrValue::Bool(true));
        assert_eq!(lit.value(), &UiIrExpression::bool(true));

        let txt = UiIrProperty::text("label", "OK");
        assert_eq!(txt.value(), &UiIrExpression::text("OK"));
    }

    #[test]
    fn binding_constructor_is_dynamic() {
        let prop = UiIrProperty::binding("title", "page.title");
        assert_eq!(prop.name(), "title");
        assert!(prop.is_dynamic());
    }

    #[test]
    fn literal_property_is_not_dynamic() {
        let prop = UiIrProperty::literal("x", UiIrValue::Int(1));
        assert!(!prop.is_dynamic());
    }
}
