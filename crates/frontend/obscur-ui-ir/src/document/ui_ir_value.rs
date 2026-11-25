#[doc = include_str!("../../docs/api/document/ui_ir_expression.doc/UiIrValue.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum UiIrValue {
    /// Absence of a value.
    Null,
    /// A boolean value.
    Bool(bool),
    /// A signed integer value.
    Int(i64),
    /// A floating-point value. Prevents deriving `Eq`/`Hash` on this enum.
    Float(f64),
    /// An owned text value.
    Text(Box<str>),
}

impl UiIrValue {
    /// Returns whether this value is [`Null`](UiIrValue::Null).
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}
