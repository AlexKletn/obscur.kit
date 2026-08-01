Single-line (no-wrap) flex layout (spec §4.1, §4.2, §4.6).

Lays items out along a main axis chosen by `style.flex_direction`, resolves
their main sizes by distributing free space via `flex_grow` / `flex_shrink`,
positions them with `justify_content` along the main axis, and aligns them
with `align_items` along the cross axis.

# Algorithm (Phase-1 subset)

1. The main axis is horizontal for [`Row`]/[`RowReverse`] and vertical for
   [`Column`]/[`ColumnReverse`]. Container content extents honour the
   container's own explicit `width` / `height` (resolved against
   `constraints.max`); an `Auto` extent falls back to `constraints.max` minus
   border + padding. Both are clamped to be non-negative. Free space, `fr`
   growth, percentages, and `justify_content` all resolve against this extent.
2. Each item's **base main size** is `flex_basis` (resolved against the
   container main extent); else its main-axis size (`width` for a row,
   `height` for a column); else its measured natural main size (measured with
   loose constraints up to the container content box). Never negative. An
   **intrinsic** keyword on either source (`flex-basis: min-content`,
   `width: fit-content`) is answered by measuring the item under a zero /
   unbounded main-axis offer — see [`intrinsic_main`](super::flex_main_probe::intrinsic_main).

   Two neighbours of that path still degrade an intrinsic keyword to `auto`:
   an item's **cross**-axis size and bounds (step 5 /
   [`clamp_cross`](super::flex_axis_bounds::clamp_cross)), and
   the **container's own** `width` / `height` and `min-`/`max-` bounds in
   step 1 — `display: flex; width: min-content` fills its parent. Both want a
   min-content size *of a flex line*, which nothing here computes: the item
   probes above answer for one item, not for the line they share.
3. `free = container_main - (sum of base sizes) - gap * (count - 1)`.
4. If `free > 0` and `sum(flex_grow) > 0`, each item grows by
   `free * grow_i / sum(grow)`. If `free < 0` and the scaled shrink factor
   `sum(flex_shrink_i * base_i) > 0`, each item shrinks proportionally to
   `flex_shrink_i * base_i`, clamped so no final main size is negative.
   Otherwise base sizes are kept and the slack is handled by justification.
5. Cross size per item is the container cross extent when
   `align_items == Stretch` (unless the item sets an explicit cross-axis size,
   which wins over the stretch), else the item's measured cross size (measured
   with its final main size fixed and a loose cross axis).
6. `justify_content` turns the leftover main-axis space into a leading offset
   and an inter-item spacing (`FlexStart`, `FlexEnd`, `Center`,
   `SpaceBetween`, `SpaceAround`, `SpaceEvenly`); when nothing is left over
   the leading offset and spacing are zero.
7. Each item is positioned along the main axis (leading + accumulated sizes +
   gaps + justify spacing) and along the cross axis per `align_items`
   (`FlexStart` → 0, `FlexEnd` → `container_cross - item_cross`, `Center` →
   half, `Stretch` → 0 filling the cross axis). Reverse directions place items
   in reversed order along the main axis.
8. Main/cross coordinates map back to x/y and are offset into the content box.
   The node border-box size is the container content size plus border and
   padding, then clamped into `constraints`.

[`Row`]: crate::model::FlexDirection::Row
[`RowReverse`]: crate::model::FlexDirection::RowReverse
[`Column`]: crate::model::FlexDirection::Column
[`ColumnReverse`]: crate::model::FlexDirection::ColumnReverse

# Out of scope for Phase 1

This is deliberately a **subset**: no wrapping / multi-line flex, no `order`
reordering, no baseline alignment (`AlignItems::Baseline` is not modelled),
and no `align-self` per-item cross override. These belong to later phases.

Per-item **main-axis min/max clamping** (`min-width`/`max-width` in a row,
the height bounds in a column) and **margins** (which consume main-axis space
and offset each item) *are* honoured; cross-axis min/max remains later work.

# `auto` margins take the free space before `justify-content` sees it

CSS Flexbox §8.1: when a line has positive free space and any main-axis margin
on it is `auto`, that space is split equally among the `auto` margins and
`justify-content` gets none. The ordering *is* the feature — it is why
`margin-left: auto` pushes an item to the end of a row no matter what
`justify-content` says, and why `justify-content` appears to stop working the
moment an `auto` margin is added. See
[`place_line`](super::flex_line_placement::place_line).

**Cross-axis `auto` margins are not honoured**: §8.1 gives them the same job in
the cross axis, where they would absorb the line's leftover cross space and
override `align-self`. Only the main axis distributes here, so a cross-axis
`auto` margin reserves nothing and alignment decides alone.
