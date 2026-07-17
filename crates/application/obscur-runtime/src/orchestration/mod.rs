//! Orchestration module.
mod animation_binding;
pub use animation_binding::{AnimatedKind, AnimationBinding};
mod css_motion_rules;
pub use css_motion_rules::{CssKeyframeRule, CssMotionRules, CssTransitionRule};
mod drag_coordinator;
pub use drag_coordinator::{DragCoordinator, DragOutcome};
mod editing_view;
pub use editing_view::{EditingView, PreeditView};
mod font_families;
pub(crate) use font_families::families_for;
mod grid_areas;
pub use grid_areas::resolve_named_grid_areas;
mod grid_lines;
pub use grid_lines::resolve_named_grid_lines;
mod grid_templates;
pub use grid_templates::{GridTemplates, grid_template_for, grid_templates_from};
mod inspect;
pub use inspect::{InspectInfo, NodeInspection, TreeInspection, overlay_commands};
mod layout_coordinator;
pub use layout_coordinator::{
    ArrangeCache, LayoutBoxes, LayoutCoordinator, LayoutStyles, MeasureCache, root_font_size,
};
mod motion_coordinator;
pub use motion_coordinator::MotionCoordinator;
mod pseudo_states;
pub use pseudo_states::PseudoStates;
mod render_coordinator;
pub use render_coordinator::RenderCoordinator;
mod scene_coordinator;
pub use scene_coordinator::{SceneCoordinator, ScrollPaint};
mod scroll;
mod shaped_ch_metrics;
pub(crate) use scroll::{ScrollContainer, overflow_scrolls, scroll_containers};
pub use shaped_ch_metrics::ShapedChMetrics;
mod stacking;
mod style_coordinator;
pub use style_coordinator::{ComputedStyles, IncrementalRestyle, StyleCoordinator};
mod style_inputs;
pub use style_inputs::StyleInputs;
mod style_matching_coordinator;
pub use style_matching_coordinator::StyleMatchingCoordinator;
mod text_attributes;
pub(crate) use text_attributes::{
    TabSize, alignment_for, attributes_for, direction_for, display_text, letter_spacing_for,
    line_height_for, overflow_for, overflow_wrap_for, tab_size_for, text_transform_for,
    white_space_for, word_break_for, word_spacing_for,
};
mod text_coordinator;
pub use text_coordinator::TextCoordinator;
mod tree_element;
pub use tree_element::TreeElement;
