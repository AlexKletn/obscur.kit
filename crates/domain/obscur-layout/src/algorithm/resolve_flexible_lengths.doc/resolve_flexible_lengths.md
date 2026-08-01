Resolves the final main sizes of one flex line's items with the canonical
**resolve flexible lengths** algorithm (CSS Flexbox Level 1, §9.7,
<https://drafts.csswg.org/css-flexbox-1/#resolve-flexible-lengths>).

This replaces a single-pass `size += free · factor / Σfactor` split, which is
wrong whenever an item clamps at its min/max (the violation must be frozen and
the freed space redistributed) or when `Σ`flex factors `< 1` (the line must be
left partly unfilled). The loop freezes items as they reach their limits and
re-distributes the remaining free space among the still-flexible ones until
every item is frozen.

`non_flex_main` is the main-axis space the gaps and item margins consume; it is
subtracted from the container inner main size once, since those extents never
flex. `flex_base` is each item's unclamped flex base size (what grow/shrink
distribute from) and `hypothetical` is that base clamped to the item's content
min/max (the freeze-inflexible reference). `main_min` / `main_max` are the
per-item pre-resolved main-axis bounds (the §4.5 automatic minimum folded into
`main_min`). All are indexed by the global item index; the returned vector is
indexed by position within `line`.
