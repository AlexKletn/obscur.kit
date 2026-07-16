//! Computed module.
mod computed_style;

pub use computed_style::ComputedStyle;

mod computed_value;
pub use computed_value::ComputedValue;
mod absolutize;
mod ch_metrics;
pub use ch_metrics::ChMetrics;
mod style_resolver;
pub use style_resolver::{ResolutionContext, StyleResolver};
mod style_diff;
pub use style_diff::StyleDiff;
mod style_snapshot;
pub use style_snapshot::StyleSnapshot;
