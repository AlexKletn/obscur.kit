#![doc = include_str!("mod.doc/_module.md")]
mod message_key;
pub use message_key::MessageKey;
mod grammatical_gender;
pub use grammatical_gender::GrammaticalGender;
mod grammatical_case;
pub use grammatical_case::GrammaticalCase;
mod message_value;
pub use message_value::MessageValue;
mod message_args;
pub use message_args::MessageArgs;
mod message_pattern_error;
pub use message_pattern_error::MessagePatternError;
mod message_pattern;
pub use message_pattern::MessagePattern;
