//! Path module.
mod fill_rule;
pub use fill_rule::FillRule;
mod path;
pub use path::Path;
mod path_data;
mod path_data_cursor;
pub use path_data::parse_path_data;
mod path_data_error;
pub use path_data_error::PathDataError;
mod path_data_fault;
pub use path_data_fault::PathDataFault;
mod path_trim;
pub use path_trim::PathTrim;
mod path_segment;
pub use path_segment::PathSegment;

mod svg_arc;
