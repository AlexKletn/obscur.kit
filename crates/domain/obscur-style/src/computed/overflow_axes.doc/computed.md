Applies css-overflow-3 §3's pairing rule to a specified `(inline, block)` pair:
if exactly one of the two is `visible` or `clip`, that one computes to `auto` or
`hidden` respectively. Exposed so a caller holding the two values already (a
test, or a phase that never had a [`ComputedStyle`](crate::computed::ComputedStyle))
can reach the same answer as [`OverflowAxes::of`].
