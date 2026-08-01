#![doc = include_str!("grid_item_alignment.doc/_module.md")]

use crate::model::{AlignItems, Dimension};

#[doc = include_str!("grid_item_alignment.doc/CellAlign.md")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CellAlign {
    #[doc = include_str!("grid_item_alignment.doc/Stretch.md")]
    Stretch,
    #[doc = include_str!("grid_item_alignment.doc/Start.md")]
    Start,
    #[doc = include_str!("grid_item_alignment.doc/End.md")]
    End,
    #[doc = include_str!("grid_item_alignment.doc/Center.md")]
    Center,
}

#[doc = include_str!("grid_item_alignment.doc/cell_align.md")]
const fn cell_align(align: AlignItems) -> CellAlign {
    match align {
        AlignItems::Stretch => CellAlign::Stretch,
        AlignItems::Center => CellAlign::Center,
        AlignItems::FlexStart | AlignItems::Start => CellAlign::Start,
        AlignItems::FlexEnd | AlignItems::End => CellAlign::End,
    }
}

#[doc = include_str!("grid_item_alignment.doc/item_axis_align.md")]
pub(super) const fn item_axis_align(align: AlignItems, size: Dimension) -> CellAlign {
    let resolved = cell_align(align);
    if matches!(resolved, CellAlign::Stretch) && !size.is_auto() {
        CellAlign::Start
    } else {
        resolved
    }
}

#[doc = include_str!("grid_item_alignment.doc/align_offset.md")]
pub(super) fn align_offset(align: CellAlign, cell: f32, item: f32) -> f32 {
    match align {
        CellAlign::Stretch | CellAlign::Start => 0.0,
        CellAlign::End => cell - item,
        CellAlign::Center => (cell - item) / 2.0,
    }
}
