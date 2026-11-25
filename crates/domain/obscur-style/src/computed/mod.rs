//! Computed module.
mod computed_style;

pub use computed_style::ComputedStyle;

mod computed_value;
pub use computed_value::ComputedValue;
mod absolutize;
pub(crate) use absolutize::{absolute_font_size, absolutize_value};
mod ch_metrics;
pub use ch_metrics::ChMetrics;
mod computed_property_set;
mod custom_value_parser;
pub use custom_value_parser::CustomValueParser;
mod effective_custom_properties;
pub(crate) use effective_custom_properties::substitute;
mod initial_value;
pub(crate) use initial_value::initial_value_of;
mod logical_cascade;
mod overflow_axes;
pub use overflow_axes::OverflowAxes;
mod relative_font_weight;
mod resolution_context;
mod substituted_value;
pub use resolution_context::ResolutionContext;
pub(crate) use substituted_value::substituted_value;
mod style_resolver;
pub use style_resolver::StyleResolver;
mod style_diff;
pub use style_diff::StyleDiff;
mod style_snapshot;
pub use style_snapshot::StyleSnapshot;
