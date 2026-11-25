//! Selection module.
mod caret;
pub use caret::Caret;
mod caret_affinity;
pub use caret_affinity::CaretAffinity;
mod selection_geometry;
pub use selection_geometry::SelectionGeometry;
mod text_selection;
pub use text_selection::TextSelection;
