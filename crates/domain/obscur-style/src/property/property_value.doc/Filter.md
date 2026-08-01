An ordered `filter` op list, in author order: the value of `filter` reduced
to the functions the pipeline can honor. Each entry is a [`FilterOp`] —
either a `drop-shadow(...)` (painted BEHIND the element like a `box-shadow`
drop shadow) or a `blur(<length>)` (a real gaussian over the element's
composited subtree on the GPU backend). Order is preserved. Other filter
functions (`brightness()`, `contrast()`, color-matrix, …) are out of scope
and dropped at lower time; `filter: none` / empty lowers to an **empty**
list (painting nothing).
