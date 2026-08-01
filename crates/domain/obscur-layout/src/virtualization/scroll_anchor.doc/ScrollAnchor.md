Preserves scroll position across item insertions/resizes by anchoring to a
specific item (spec §4.9 *scroll anchoring*).

The anchor captures the item at the viewport top ([`index`](Self::index)) plus
[`offset_within_item`](Self::offset_within_item) — how far into that item the
top sits — and the [`viewport_offset`](Self::viewport_offset) it was captured
at (kept for reference). [`resolve`](Self::resolve) recomputes the scroll
offset from current extents so the anchored item stays put even after items
above it change size.
