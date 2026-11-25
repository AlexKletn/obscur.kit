use obscur_style::cascade::Declaration;
use obscur_style::matching::StyleRule;
use obscur_style::property::CustomProperty;

use crate::error::UnsupportedCssFeature;

use super::{
    CompiledConditionalGroup, CompiledKeyframes, CompiledTransition,
    pending_motion_rule::PendingMotionRule,
};

/// The staging buffer a rule list compiles into before it is lifted into the
/// [`CompiledStylesheet`] (or wrapped in a [`CompiledConditionalGroup`]).
#[derive(Default)]
pub(super) struct StylesheetAccumulator {
    pub(super) declarations: Vec<Declaration>,
    pub(super) custom_properties: Vec<CustomProperty>,
    pub(super) conditional_groups: Vec<CompiledConditionalGroup>,
    pub(super) keyframes: Vec<CompiledKeyframes>,
    pub(super) transitions: Vec<CompiledTransition>,
    pub(super) matchable_rules: Vec<StyleRule>,
    pub(super) motion_rules: Vec<PendingMotionRule>,
    pub(super) unsupported: Vec<UnsupportedCssFeature>,
}
