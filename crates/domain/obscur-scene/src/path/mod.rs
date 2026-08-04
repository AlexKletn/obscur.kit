#![doc = include_str!("mod.doc/_module.md")]
mod fill_rule;
pub use fill_rule::FillRule;
mod path;
pub use path::Path;
mod path_data;
pub use path_data::{PathDataError, PathDataFault, parse_path_data};
mod path_trim;
pub use path_trim::PathTrim;
mod path_segment;
pub use path_segment::PathSegment;
