//! `MessageValue`: one argument bound into a message (spec §7.2 parameters).

use crate::message::{GrammaticalCase, GrammaticalGender};
use crate::plural::PluralOperands;

#[doc = include_str!("../../docs/api/message/message_value.doc/MessageValue.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum MessageValue {
    /// A literal string, interpolated as-is.
    Text(Box<str>),
    /// A number: the digits to interpolate, plus the operands its category is
    /// selected from.
    Number {
        /// How the number is written, and what `{name}` or `#` interpolates.
        literal: Box<str>,
        /// The CLDR operands `literal` denotes, which a `plural` arm selects on.
        operands: PluralOperands,
    },
    /// A selector matched against `select` arm names — a gender, a case, or any
    /// other closed-by-the-catalogue choice.
    Selector(Box<str>),
}

impl MessageValue {
    #[doc = include_str!("../../docs/api/message/message_value.doc/integer.md")]
    #[must_use]
    pub fn integer(count: i64) -> Self {
        Self::Number {
            literal: count.to_string().into_boxed_str(),
            operands: PluralOperands::from_integer(count),
        }
    }

    #[doc = include_str!("../../docs/api/message/message_value.doc/decimal.md")]
    #[must_use]
    pub fn decimal(decimal: &str) -> Option<Self> {
        Some(Self::Number {
            literal: decimal.into(),
            operands: PluralOperands::from_decimal(decimal)?,
        })
    }

    /// Binds a literal string.
    #[must_use]
    pub fn text(text: impl Into<Box<str>>) -> Self {
        Self::Text(text.into())
    }

    /// Binds a bare selector value.
    #[must_use]
    pub fn selector(value: impl Into<Box<str>>) -> Self {
        Self::Selector(value.into())
    }

    /// Binds a grammatical gender as a selector (spec §7.2 gender).
    #[must_use]
    pub fn gender(gender: &GrammaticalGender) -> Self {
        Self::Selector(gender.as_str().into())
    }

    /// Binds a grammatical case as a selector (spec §7.2 case variants).
    #[must_use]
    pub fn grammatical_case(case: &GrammaticalCase) -> Self {
        Self::Selector(case.as_str().into())
    }

    #[doc = include_str!("../../docs/api/message/message_value.doc/literal_2.md")]
    #[must_use]
    pub fn literal(&self) -> &str {
        match self {
            Self::Text(text) | Self::Selector(text) => text,
            Self::Number { literal, .. } => literal,
        }
    }

    #[doc = include_str!("../../docs/api/message/message_value.doc/operands_2.md")]
    #[must_use]
    pub const fn operands(&self) -> Option<&PluralOperands> {
        match self {
            Self::Number { operands, .. } => Some(operands),
            _ => None,
        }
    }

    #[doc = include_str!("../../docs/api/message/message_value.doc/selector_name.md")]
    #[must_use]
    pub fn selector_name(&self) -> &str {
        self.literal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::LanguageTag;
    use crate::plural::PluralCategory;
    use crate::plural::PluralRules;

    #[test]
    fn an_integer_carries_both_its_digits_and_its_operands() {
        let value = MessageValue::integer(21);
        assert_eq!(value.literal(), "21");
        assert_eq!(value.operands().expect("number").i, 21);
    }

    #[test]
    fn a_decimal_keeps_the_spelling_it_was_given() {
        let value = MessageValue::decimal("2.50").expect("valid");
        assert_eq!(value.literal(), "2.50");
        let operands = value.operands().expect("number");
        assert_eq!((operands.v, operands.w), (2, 1));
    }

    #[test]
    fn a_decimal_refuses_a_spelling_it_cannot_read() {
        assert!(MessageValue::decimal("1e6").is_none());
        assert!(MessageValue::decimal("1 000").is_none());
    }

    #[test]
    fn the_operands_a_number_carries_drive_a_real_category() {
        // The binding is not decorative: it is what the rules select on.
        let ru = PluralRules::cardinal(&LanguageTag::parse("ru").expect("tag"));
        let value = MessageValue::integer(111);
        assert_eq!(
            ru.select(value.operands().expect("number")),
            PluralCategory::Many
        );
    }

    #[test]
    fn text_and_selectors_have_no_operands() {
        assert!(MessageValue::text("Мария").operands().is_none());
        assert!(MessageValue::selector("feminine").operands().is_none());
    }

    #[test]
    fn gender_and_case_bind_as_selectors() {
        let g = MessageValue::gender(&GrammaticalGender::feminine());
        assert_eq!(g.selector_name(), "feminine");
        let c = MessageValue::grammatical_case(&GrammaticalCase::genitive());
        assert_eq!(c.selector_name(), "genitive");
    }

    #[test]
    fn every_variant_interpolates_to_something() {
        assert_eq!(MessageValue::text("файл").literal(), "файл");
        assert_eq!(MessageValue::selector("feminine").literal(), "feminine");
        assert_eq!(MessageValue::integer(-5).literal(), "-5");
    }
}
