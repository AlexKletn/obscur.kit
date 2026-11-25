//! Orchestration module.
mod animation_binding;
pub use animation_binding::AnimationBinding;
mod animated_kind;
pub use animated_kind::AnimatedKind;
mod arrange_cache;
mod arrange_key;
pub use arrange_cache::ArrangeCache;
mod arrange_pass;
mod background_paint;
mod border_paint;
pub(crate) mod clip_path;
pub(crate) use border_paint::corner_radii_of;
pub(crate) use border_paint::{overflow_padding_box, overflow_padding_radii};
mod box_shadow_paint;
mod caret_geometry;
pub(crate) use caret_geometry::{
    caret_line_move, caret_line_scroll_geometry, caret_offset_at_point, caret_visual_line_bound,
    line_caret_x, line_of_byte,
};
mod composite_effects;
mod css_keyframe_rule;
mod css_motion_rules;
mod css_transition_rule;
pub use css_keyframe_rule::CssKeyframeRule;
pub use css_motion_rules::CssMotionRules;
pub use css_transition_rule::CssTransitionRule;
mod drag_coordinator;
pub use drag_coordinator::DragCoordinator;
mod drag_outcome;
pub use drag_outcome::DragOutcome;
mod editing_role;
pub(crate) use editing_role::editing_override;
mod editing_view;
pub use editing_view::EditingView;
mod preedit_view;
pub use preedit_view::PreeditView;
mod inline_style_outcome;
pub use inline_style_outcome::InlineStyleOutcome;
mod inline_styles;
pub use inline_styles::InlineStyleCompiler;
pub(crate) use inline_styles::InlineStyles;
mod filter_color_matrix;
mod font_families;
mod formatting_context;
pub(crate) use font_families::families_for;
mod gradient_brush;
mod grid_areas;
pub use grid_areas::resolve_named_grid_areas;
mod grid_lines;
pub(crate) use grid_lines::defer_repeated_grid_lines;
pub use grid_lines::resolve_named_grid_lines;
mod grid_templates;
pub use grid_templates::{GridTemplates, grid_template_for, grid_templates_from};
mod inspect;
pub use inspect::overlay_commands;
mod inspect_info;
pub use inspect_info::InspectInfo;
mod node_inspection;
pub use node_inspection::NodeInspection;
mod tree_inspection;
pub use tree_inspection::TreeInspection;
mod layout_coordinator;
pub use layout_coordinator::{LayoutBoxes, LayoutCoordinator, LayoutStyles};
mod layout_eviction;
mod layout_style_mapping;
mod layout_topology;
pub(crate) use layout_style_mapping::{layout_style_from, writing_mode_of};
mod length_resolution;
pub use length_resolution::root_font_size;
mod measure_cache;
pub use measure_cache::MeasureCache;
mod measure_pass;
mod motion_coordinator;
pub use motion_coordinator::MotionCoordinator;
mod motion_playback;
pub use motion_playback::MotionPlayback;
mod paint_color;
mod paint_transform;
pub(crate) use paint_transform::{
    about_transform_origin, resolved_transform, transform_matrix, transform_origin,
};
mod paragraph_shaping;
mod pseudo_states;
pub use pseudo_states::PseudoStates;
mod render_coordinator;
pub use render_coordinator::RenderCoordinator;
mod replaced_content;
mod scene_coordinator;
pub use scene_coordinator::SceneCoordinator;
mod scroll;
mod scroll_container_paint;
pub(crate) use scroll_container_paint::{
    horizontal_scrollbar_rects, horizontal_thumb_offset, scrollbar_rects,
};
mod scroll_paint;
pub use scroll_paint::ScrollPaint;
mod shaped_ch_metrics;
pub(crate) use scroll::{
    ScrollContainer, establishes_scroll_container, overflow_scrolls, overflow_scrolls_x,
    scroll_containers,
};
pub use shaped_ch_metrics::ShapedChMetrics;
mod context_entry;
mod effective_stacking;
mod paint_layer;
mod paint_unit;
mod stacking;
pub(crate) use effective_stacking::EffectiveStacking;
pub(crate) use stacking::effective_stacking;
mod style_coordinator;
pub use style_coordinator::{ComputedStyles, CustomValues, StyleCoordinator};
mod incremental_restyle;
pub use incremental_restyle::IncrementalRestyle;
mod style_inputs;
pub use style_inputs::StyleInputs;
mod lazy_rule_index;
mod style_matching_coordinator;
pub use style_matching_coordinator::StyleMatchingCoordinator;
mod style_scoping;
pub(crate) use style_scoping::{component_host, part_specificity_for, subject_part_name};
mod generated_boxes;
pub(crate) use generated_boxes::GeneratedBoxes;
mod candidate_rule;
mod generated_content;
mod pseudo_rule_index;
pub use generated_content::GeneratedContent;
pub(crate) use generated_content::resolve_generated_content;
mod generated_run;
pub use generated_run::GeneratedRun;
mod subtree_brackets;
mod tab_size;
pub(crate) use tab_size::TabSize;
mod text_attributes;
pub(crate) use text_attributes::{
    alignment_for, attributes_for, direction_for, display_text, letter_spacing_for,
    line_height_for, overflow_for, overflow_wrap_for, tab_size_for, text_transform_for,
    white_space_for, word_break_for, word_spacing_for,
};
mod text_coordinator;
pub use text_coordinator::TextCoordinator;
mod text_leaf_measure;
mod text_painter;
pub(crate) use text_painter::CARET_WIDTH;
mod tree_element;
pub use tree_element::TreeElement;
