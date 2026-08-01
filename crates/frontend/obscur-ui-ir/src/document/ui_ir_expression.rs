#![doc = include_str!("ui_ir_expression.doc/_module.md")]

#[doc = include_str!("ui_ir_expression.doc/UiIrValue.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum UiIrValue {
    #[doc = include_str!("ui_ir_expression.doc/Null.md")]
    Null,
    #[doc = include_str!("ui_ir_expression.doc/Bool.md")]
    Bool(bool),
    #[doc = include_str!("ui_ir_expression.doc/Int.md")]
    Int(i64),
    #[doc = include_str!("ui_ir_expression.doc/Float.md")]
    Float(f64),
    #[doc = include_str!("ui_ir_expression.doc/Text.md")]
    Text(Box<str>),
}

impl UiIrValue {
    #[doc = include_str!("ui_ir_expression.doc/is_null.md")]
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

#[doc = include_str!("ui_ir_expression.doc/UiIrExpression.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum UiIrExpression {
    #[doc = include_str!("ui_ir_expression.doc/Literal.md")]
    Literal(UiIrValue),
    #[doc = include_str!("ui_ir_expression.doc/Binding.md")]
    Binding(Box<str>),
    #[doc = include_str!("ui_ir_expression.doc/Call.md")]
    Call {
        #[doc = include_str!("ui_ir_expression.doc/name.md")]
        name: Box<str>,
        #[doc = include_str!("ui_ir_expression.doc/args.md")]
        args: Vec<UiIrExpression>,
    },
}

impl UiIrExpression {
    #[doc = include_str!("ui_ir_expression.doc/literal.md")]
    #[must_use]
    pub const fn literal(value: UiIrValue) -> Self {
        Self::Literal(value)
    }

    #[doc = include_str!("ui_ir_expression.doc/text.md")]
    #[must_use]
    pub fn text(value: impl Into<Box<str>>) -> Self {
        Self::Literal(UiIrValue::Text(value.into()))
    }

    #[doc = include_str!("ui_ir_expression.doc/bool.md")]
    #[must_use]
    pub const fn bool(value: bool) -> Self {
        Self::Literal(UiIrValue::Bool(value))
    }

    #[doc = include_str!("ui_ir_expression.doc/int.md")]
    #[must_use]
    pub const fn int(value: i64) -> Self {
        Self::Literal(UiIrValue::Int(value))
    }

    #[doc = include_str!("ui_ir_expression.doc/binding.md")]
    #[must_use]
    pub fn binding(path: impl Into<Box<str>>) -> Self {
        Self::Binding(path.into())
    }

    #[doc = include_str!("ui_ir_expression.doc/call.md")]
    #[must_use]
    pub fn call(name: impl Into<Box<str>>, args: Vec<UiIrExpression>) -> Self {
        Self::Call {
            name: name.into(),
            args,
        }
    }

    #[doc = include_str!("ui_ir_expression.doc/is_literal.md")]
    #[must_use]
    pub const fn is_literal(&self) -> bool {
        matches!(self, Self::Literal(_))
    }

    #[doc = include_str!("ui_ir_expression.doc/as_literal.md")]
    #[must_use]
    pub const fn as_literal(&self) -> Option<&UiIrValue> {
        match self {
            Self::Literal(value) => Some(value),
            Self::Binding(_) | Self::Call { .. } => None,
        }
    }

    #[doc = include_str!("ui_ir_expression.doc/as_binding.md")]
    #[must_use]
    pub fn as_binding(&self) -> Option<&str> {
        match self {
            Self::Binding(path) => Some(path),
            Self::Literal(_) | Self::Call { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_constructors_build_expected_values() {
        assert_eq!(
            UiIrExpression::text("hi"),
            UiIrExpression::Literal(UiIrValue::Text("hi".into()))
        );
        assert_eq!(
            UiIrExpression::bool(true),
            UiIrExpression::Literal(UiIrValue::Bool(true))
        );
        assert_eq!(
            UiIrExpression::int(7),
            UiIrExpression::Literal(UiIrValue::Int(7))
        );
        assert_eq!(
            UiIrExpression::literal(UiIrValue::Null),
            UiIrExpression::Literal(UiIrValue::Null)
        );
    }

    #[test]
    fn is_null_reports_null_variant() {
        assert!(UiIrValue::Null.is_null());
        assert!(!UiIrValue::Bool(false).is_null());
    }

    #[test]
    fn binding_holds_path() {
        let expr = UiIrExpression::binding("user.name");
        assert_eq!(expr, UiIrExpression::Binding("user.name".into()));
        assert_eq!(expr.as_binding(), Some("user.name"));
    }

    #[test]
    fn call_holds_name_and_args() {
        let expr = UiIrExpression::call("upper", vec![UiIrExpression::text("hi")]);
        assert_eq!(
            expr,
            UiIrExpression::Call {
                name: "upper".into(),
                args: vec![UiIrExpression::Literal(UiIrValue::Text("hi".into()))],
            }
        );
    }

    #[test]
    fn is_literal_and_as_literal() {
        let lit = UiIrExpression::int(5);
        assert!(lit.is_literal());
        assert_eq!(lit.as_literal(), Some(&UiIrValue::Int(5)));

        let bind = UiIrExpression::binding("a.b");
        assert!(!bind.is_literal());
        assert_eq!(bind.as_literal(), None);
    }

    #[test]
    fn as_binding_is_none_for_non_binding() {
        assert_eq!(UiIrExpression::int(1).as_binding(), None);
        assert_eq!(UiIrExpression::call("f", Vec::new()).as_binding(), None);
    }
}
