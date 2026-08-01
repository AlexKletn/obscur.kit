Optional lower and upper bounds on a window's size (spec §6.2).

Either bound may be absent (`None`), leaving that side unconstrained. The
default is fully unconstrained (both `None`). Use
[`WindowConstraints::constrain`] to clamp a candidate size into the
permitted range.
