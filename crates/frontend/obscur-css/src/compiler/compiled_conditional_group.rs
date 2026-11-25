use obscur_style::cascade::Declaration;
use obscur_style::property::CustomProperty;

use super::{CompiledCondition, CompiledTransition};

#[doc = include_str!("../../docs/api/compiler/stylesheet_compiler.doc/CompiledConditionalGroup.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledConditionalGroup {
    /// The condition under which the grouped declarations apply.
    pub condition: CompiledCondition,
    /// The standard declarations inside the conditional block.
    pub declarations: Vec<Declaration>,
    /// The custom properties inside the conditional block.
    pub custom_properties: Vec<CustomProperty>,
    /// The transition templates inside the conditional block.
    pub transitions: Vec<CompiledTransition>,
}
