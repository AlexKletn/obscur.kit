The rule: with the parent's height `auto` the containing block is indefinite,
so both percentages behave as `auto` and an empty block's `auto` height is its
content — zero.

Measured by reverting the fix, this read **`(225, 112.5)`**: the first child
took half of the 450 it was offered, and the second took half of the 225 that
left — the compounding, and neither number is a percentage of anything the CSS
names.
