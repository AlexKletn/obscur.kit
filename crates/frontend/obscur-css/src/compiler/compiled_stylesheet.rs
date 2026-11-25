use obscur_style::cascade::Declaration;
use obscur_style::matching::Stylesheet;
use obscur_style::property::CustomProperty;

use crate::error::{CssParseError, UnsupportedCssFeature};

use super::{
    CompiledConditionalGroup, CompiledKeyframes, CompiledMotionRule, CompiledTransition,
    CssSymbolInterner,
};

/// The fully compiled stylesheet: the native structures the rest of the kit
/// consumes (spec: "Выход — структуры obscur-style и obscur-motion").
#[derive(Debug, Clone)]
pub struct CompiledStylesheet {
    /// Top-level standard cascade declarations, in source order.
    pub declarations: Vec<Declaration>,
    /// Top-level custom-property declarations.
    pub custom_properties: Vec<CustomProperty>,
    /// Conditional (`@media` / `@container`) groups.
    pub conditional_groups: Vec<CompiledConditionalGroup>,
    /// Compiled `@keyframes` rules.
    pub keyframes: Vec<CompiledKeyframes>,
    /// Top-level `transition` templates.
    pub transitions: Vec<CompiledTransition>,
    #[doc = include_str!("../../docs/api/compiler/stylesheet_compiler.doc/motion_rules.md")]
    pub motion_rules: Vec<CompiledMotionRule>,
    /// The interner backing every [`InternedSymbol`](obscur_identity::InternedSymbol) in this stylesheet.
    pub symbols: CssSymbolInterner,
    #[doc = include_str!("../../docs/api/compiler/stylesheet_compiler.doc/matchable.md")]
    pub matchable: Stylesheet,
    /// Recognised-but-unsupported features (from parsing and compiling).
    pub unsupported: Vec<UnsupportedCssFeature>,
    /// Syntax errors carried forward from parsing.
    pub errors: Vec<CssParseError>,
}
