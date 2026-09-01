Whether an axis's margins claim the item's free space in its grid area, which
they do as soon as either edge is `auto`.

Asked before the item is measured, because the answer decides whether the item
stretches at all: an `auto` margin makes it auto-size to its contents
(`specs/w3c/css-grid-2.txt:3328`). Asked again after, because an axis that
answers `true` here is placed by [`auto_lead`] and not by
[`align_offset`](super::grid_item_alignment::align_offset) — the free space is
spent once, by whichever of the two owns it.
