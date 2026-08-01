The **effective** cross-axis alignment for one item: its own `align-self`,
falling back to the container's `align-items` when `auto` (the initial value).

Flex previously read the container's `align-items` for every item, so a
per-item `align-self` was silently dropped — while grid honoured it. This is
the same `AlignSelf::resolve` grid uses, so both contexts now agree.
