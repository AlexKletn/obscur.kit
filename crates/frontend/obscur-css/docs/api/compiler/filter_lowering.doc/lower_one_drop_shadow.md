Lowers one `drop-shadow(<offset-x> <offset-y> <blur>? <color>?)` into a drop
[`BoxShadow`] (`spread == 0`, `inset == false`).

A drop-shadow is the `box-shadow` DROP subset: two required `<length>` offsets,
an optional third `<length>` blur radius (spread and `inset` are not part of the
`drop-shadow()` grammar), and an optional `<color>` defaulting to `currentColor`
(kept as [`None`], resolved at paint time — mirroring `box-shadow`). The
arguments are **space-separated** (not comma-separated). Returns [`None`] for a
duplicate color, a stray comma, an unparseable component, or fewer than two /
more than three lengths.
