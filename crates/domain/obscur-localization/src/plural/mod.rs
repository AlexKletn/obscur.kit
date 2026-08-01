#![doc = include_str!("mod.doc/_module.md")]
mod plural_category;
pub use plural_category::PluralCategory;
mod plural_operands;
pub use plural_operands::PluralOperands;
mod plural_rules;
pub use plural_rules::PluralRules;
