Sizes, justifies, and cross-aligns one flex line, writing each item's box and
returning the line's cross extent (spec §4.2, §4.6).

Distributes the line's main-axis free space via `flex_grow` / `flex_shrink`,
re-clamps into each item's main min/max range, positions items with
`justify_content` along the main axis, and aligns them with `align_items`
along the cross axis. Reverse directions place items in reversed main order.
