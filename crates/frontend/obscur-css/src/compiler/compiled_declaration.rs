use crate::error::UnsupportedCssFeature;
use obscur_style::{cascade::Declaration, property::CustomProperty};

#[doc = include_str!("../../docs/api/compiler/property_compiler.doc/CompiledDeclaration.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum CompiledDeclaration {
    /// A standard longhand declaration ready for the cascade.
    Standard(Declaration),
    /// A `--`-prefixed custom property, carrying the text it was declared with
    /// rather than a lowered value — see the `custom_value` module for why, and for
    /// the parse that reads it back at substitution time.
    Custom(CustomProperty),
    /// A recognised-but-unsupported property or value (§4.13).
    Unsupported(UnsupportedCssFeature),
}
