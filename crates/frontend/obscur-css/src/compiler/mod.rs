#![doc = include_str!("mod.doc/_module.md")]
mod animation_compiler;
pub use animation_compiler::{AnimationCompiler, AnimationRef};
mod background_lowering;
mod border_shorthand;
mod calc_lowering;
mod color_lowering;
mod condition_lowering;
mod custom_value;
pub use custom_value::lower_custom_value;
mod filter_lowering;
mod font_lowering;
mod gradient_lowering;
mod grid_placement;
mod grid_track_lowering;
mod inline_style_compiler;
pub use inline_style_compiler::{CompiledInlineStyle, compile_inline_style};
mod keyframe_compiler;
pub use keyframe_compiler::{CompiledKeyframes, KeyframeCompiler};
mod longhand_catalog;
mod motion_rule;
pub use motion_rule::{CompiledAnimation, CompiledMotionRule};
mod property_compiler;
pub use property_compiler::{CompiledDeclaration, PropertyCompiler};
mod selector_compiler;
pub use selector_compiler::{CompiledSelector, SelectorCompiler};
mod shadow_lowering;
mod shorthand_expansion;
mod stylesheet_compiler;
pub use stylesheet_compiler::{
    CompiledCondition, CompiledConditionalGroup, CompiledStylesheet, CssSymbolInterner,
    StylesheetCompiler,
};
mod transform_lowering;
mod transition_compiler;
pub use transition_compiler::{CompiledTransition, TransitionCompiler};
mod value_lowering;
mod vendor_prefix;
