#![doc = include_str!("mod.doc/_module.md")]
mod computed_style;

pub use computed_style::ComputedStyle;

mod computed_value;
pub use computed_value::ComputedValue;
mod absolutize;
mod ch_metrics;
pub use ch_metrics::ChMetrics;
mod computed_property_set;
mod effective_custom_properties;
mod initial_value;
mod logical_cascade;
mod relative_font_weight;
mod resolution_context;
pub use resolution_context::ResolutionContext;
mod style_resolver;
pub use style_resolver::StyleResolver;
mod style_diff;
pub use style_diff::StyleDiff;
mod style_snapshot;
pub use style_snapshot::StyleSnapshot;
