One `(bound, measured)` pair as a **content**-axis length, or [`None`] when it
imposes no limit (an `auto` bound, or an intrinsic one the caller could not
measure).

# `box-sizing` applies to the authored length, not to the measured one

An authored `min-width: 200px` is a border-box length under
`box-sizing: border-box`, so it is mapped through `box_sizing` like every other
authored length. `min-width: min-content` authors no length at all: what came
back from measuring is already a content width, so there is nothing for
`box-sizing` to reinterpret — the same split
[`intrinsic_inline_size`](super::intrinsic_inline_size) makes for an intrinsic
`width`.
