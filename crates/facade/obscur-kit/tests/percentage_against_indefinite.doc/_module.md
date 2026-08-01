A percentage block size against an **indefinite** containing block *behaves as
auto* — it does not resolve against whatever space happened to be offered.

`specs/w3c/css-sizing-3.txt:554` (§3.2.1 "Behaving as auto"), vendored here:

> To have a common term for both when width/height computes to auto and when it
> is defined to behave as if auto were specified (as in the case of **block
> percentage heights resolving against an indefinite size**, see CSS2§10.5),
> the property is said to **behave as auto** in both of these cases.

The defect this pins: the layout constraint carried the containing block's
content box only when the block axis was *definite*, and `percent_height()`
silently fell back to the constraint's upper bound otherwise — the space
*offered* to the child. So under an `auto`-height parent a `height: 50%` child
read half of the room its predecessors had left, and two such siblings
compounded — `(225, 112.5)` in the geometry below, measured by reverting the
fix — instead of both behaving as `auto`. An indefinite basis
is now stated as `f32::INFINITY`, which `Dimension::resolve` already answers
`None` to, and `None` is the `auto` path every caller already has.

This is the block-flow half of the percentage model. Its grid half — an item's
containing block being its grid area on both axes — is
`percentage_in_grid_item.rs`.
