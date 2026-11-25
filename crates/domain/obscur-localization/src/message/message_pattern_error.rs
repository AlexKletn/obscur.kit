//! `MessagePatternError`: why a message pattern would not parse (spec §7.2).

use std::fmt;

#[doc = include_str!("../../docs/api/message/message_pattern_error.doc/MessagePatternError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessagePatternError {
    /// A `{` was opened and never closed.
    UnclosedPlaceholder,
    /// A `}` appeared with nothing open, or a `{`/`#` appeared where a literal
    /// must not contain one (see [`MessagePattern`](crate::message::MessagePattern)
    /// on why there is no escape).
    UnexpectedBrace,
    /// A placeholder named nothing (`{}`, `{, plural, …}`).
    EmptyArgumentName,
    /// A placeholder's second field was neither `plural` nor `select`.
    UnknownSelector(Box<str>),
    /// A selector placeholder listed no arms.
    NoArms,
    /// A `plural` arm was named something that is neither a CLDR plural category
    /// nor an `=<integer>` exact match.
    UnknownPluralArm(Box<str>),
    /// A selector placeholder had no `other` arm. CLDR requires one of every rule
    /// set, and it is what makes an unmatched value renderable at all.
    MissingOtherArm,
}

impl fmt::Display for MessagePatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnclosedPlaceholder => f.write_str("unclosed `{` in message pattern"),
            Self::UnexpectedBrace => {
                f.write_str("stray `{`, `}` or `#` in message pattern (there is no escape syntax)")
            }
            Self::EmptyArgumentName => f.write_str("placeholder names no argument"),
            Self::UnknownSelector(name) => {
                write!(
                    f,
                    "unknown selector `{name}`: expected `plural` or `select`"
                )
            }
            Self::NoArms => f.write_str("selector placeholder lists no arms"),
            Self::UnknownPluralArm(name) => write!(
                f,
                "`{name}` is not a CLDR plural category or an `=<integer>` match"
            ),
            Self::MissingOtherArm => f.write_str("selector placeholder has no `other` arm"),
        }
    }
}

impl std::error::Error for MessagePatternError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_says_something_specific() {
        let messages = [
            MessagePatternError::UnclosedPlaceholder.to_string(),
            MessagePatternError::UnexpectedBrace.to_string(),
            MessagePatternError::EmptyArgumentName.to_string(),
            MessagePatternError::UnknownSelector("plurals".into()).to_string(),
            MessagePatternError::NoArms.to_string(),
            MessagePatternError::UnknownPluralArm("singular".into()).to_string(),
            MessagePatternError::MissingOtherArm.to_string(),
        ];
        for message in &messages {
            assert!(!message.is_empty());
        }
        let mut unique = messages.to_vec();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), messages.len(), "two variants read alike");
    }

    #[test]
    fn names_the_offending_token() {
        assert!(
            MessagePatternError::UnknownPluralArm("singular".into())
                .to_string()
                .contains("singular")
        );
    }
}
