//! `UiIrProperty`: a named property on a node whose value is a
//! [`UiIrExpression`] (spec §2.5).

use crate::document::{UiIrExpression, UiIrValue};

/// A named property on an IR node, flowing a [`UiIrExpression`] value *down*
/// into the node (spec §2.5).
///
/// A property pairs a name with an expression. The expression may be a baked-in
/// literal or a dynamic binding/call; [`is_dynamic`](UiIrProperty::is_dynamic)
/// reports which.
#[derive(Debug, Clone, PartialEq)]
pub struct UiIrProperty {
    name: Box<str>,
    value: UiIrExpression,
}

impl UiIrProperty {
    /// Creates a property binding `name` to `value`.
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, value: UiIrExpression) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// Creates a property whose value is a literal [`UiIrValue`].
    #[must_use]
    pub fn literal(name: impl Into<Box<str>>, value: UiIrValue) -> Self {
        Self::new(name, UiIrExpression::literal(value))
    }

    /// Creates a property whose value is a literal text expression.
    #[must_use]
    pub fn text(name: impl Into<Box<str>>, value: impl Into<Box<str>>) -> Self {
        Self::new(name, UiIrExpression::text(value))
    }

    /// Creates a property whose value is a dynamic binding to `path`.
    #[must_use]
    pub fn binding(name: impl Into<Box<str>>, path: impl Into<Box<str>>) -> Self {
        Self::new(name, UiIrExpression::binding(path))
    }

    /// Returns the property name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the property's value expression.
    #[must_use]
    pub const fn value(&self) -> &UiIrExpression {
        &self.value
    }

    /// Returns whether the value is dynamic — i.e. not a literal (a binding or
    /// call resolved at runtime).
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
