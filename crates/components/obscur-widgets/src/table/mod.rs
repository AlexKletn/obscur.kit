//! Table collection widget (spec §2.6).

mod table;
mod table_row;
pub use table::Table;
mod column;
pub use column::Column;
mod sort_direction;
pub use sort_direction::SortDirection;
