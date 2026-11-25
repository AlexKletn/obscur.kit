#![doc = include_str!("../../docs/api/algorithm/grid_item_alignment.doc/_module.md")]

use crate::model::{AlignItems, Dimension};

/// How a grid item is positioned within its cell on one axis, after resolving
/// the container default (`justify-items` / `align-items`) against any per-item
/// override (`justify-self` / `align-self`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CellAlign {
    /// Fill the cell extent (the default; measured with a fixed size).
    Stretch,
    /// Place the item at the cell start.
    Start,
    /// Place the item at the cell end (`cell − item`).
    End,
    /// Center the item within the cell (`(cell − item) / 2`).
    Center,
    /// First-baseline alignment in the block axis.
    Baseline,
    /// Last-baseline alignment in the block axis.
    LastBaseline,
}

/// Classifies an effective [`AlignItems`] into a grid [`CellAlign`]. The flexbox
/// spellings (`FlexStart` / `FlexEnd`) fold onto the grid `Start` / `End`
/// physical alignments so an authored `align-items: start` (which the runtime
/// maps to [`AlignItems::FlexStart`]) behaves as a grid start.
const fn cell_align(align: AlignItems) -> CellAlign {
    match align {
        AlignItems::Stretch => CellAlign::Stretch,
        AlignItems::Center => CellAlign::Center,
        AlignItems::Baseline => CellAlign::Baseline,
        AlignItems::LastBaseline => CellAlign::LastBaseline,
        AlignItems::FlexStart | AlignItems::Start => CellAlign::Start,
        AlignItems::FlexEnd | AlignItems::End => CellAlign::End,
    }
}

#[doc = include_str!("../../docs/api/algorithm/grid_item_alignment.doc/item_axis_align.md")]
pub(super) const fn item_axis_align(align: AlignItems, size: &Dimension) -> CellAlign {
    let resolved = cell_align(align);
    if matches!(resolved, CellAlign::Stretch) && !size.is_auto() {
        CellAlign::Start
    } else {
        resolved
    }
}

/// The offset of an `item`-sized box within a `cell`-sized extent for `align`.
/// `Stretch` and `Start` sit at the cell start; `End` at `cell − item`; `Center`
/// halves the slack.
pub(super) fn align_offset(align: CellAlign, cell: f32, item: f32) -> f32 {
    match align {
        CellAlign::Stretch | CellAlign::Start | CellAlign::Baseline => 0.0,
        CellAlign::End | CellAlign::LastBaseline => cell - item,
        CellAlign::Center => (cell - item) / 2.0,
    }
}
