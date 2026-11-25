use crate::plural::PluralCategory;

use super::MessagePattern;

/// One piece of a parsed pattern.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum PatternPart {
    /// Literal text, emitted verbatim.
    Literal(Box<str>),
    /// `{name}` — the argument's text.
    Argument(Box<str>),
    /// `#` inside a `plural` arm — the number being pluralised.
    Number,
    /// `{name, plural, …}`.
    Plural {
        /// The argument selected on.
        name: Box<str>,
        /// `=<integer>` arms, checked before the category arms.
        exact: Vec<(u64, MessagePattern)>,
        /// Category arms.
        categories: Vec<(PluralCategory, MessagePattern)>,
        /// The mandatory `other` arm.
        other: MessagePattern,
    },
    /// `{name, select, …}`.
    Select {
        /// The argument selected on.
        name: Box<str>,
        /// Named arms, in source order.
        arms: Vec<(Box<str>, MessagePattern)>,
        /// The mandatory `other` arm.
        other: MessagePattern,
    },
}
