The used `overflow` value on each axis of one box, with css-overflow-3 §3's
pairing rule already applied.

Build one with [`OverflowAxes::of`] from a node's computed style. The values are
the reserved `OVERFLOW_*` handles in [`crate::property::layout_keyword`]; an
undeclared axis is the initial `visible`.
