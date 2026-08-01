#![doc = include_str!("message_pattern_error.doc/_module.md")]

use std::fmt;

#[doc = include_str!("message_pattern_error.doc/MessagePatternError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessagePatternError {
    #[doc = include_str!("message_pattern_error.doc/UnclosedPlaceholder.md")]
    UnclosedPlaceholder,
    #[doc = include_str!("message_pattern_error.doc/UnexpectedBrace.md")]
    UnexpectedBrace,
    #[doc = include_str!("message_pattern_error.doc/EmptyArgumentName.md")]
    EmptyArgumentName,
    #[doc = include_str!("message_pattern_error.doc/UnknownSelector.md")]
    UnknownSelector(Box<str>),
    #[doc = include_str!("message_pattern_error.doc/NoArms.md")]
    NoArms,
    #[doc = include_str!("message_pattern_error.doc/UnknownPluralArm.md")]
    UnknownPluralArm(Box<str>),
    #[doc = include_str!("message_pattern_error.doc/MissingOtherArm.md")]
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
