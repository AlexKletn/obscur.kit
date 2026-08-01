The [`LengthContext`] for one node: its own `font-size` for `em`, the
document `root_font_size` for `rem`, the `viewport` for `vw`/`vh`, and
`ch_width` — the measured `0`-glyph advance — for `ch`.

Pass `ch_width: None` when the `0` glyph was not measured, either because the
node writes no `ch` (the usual case, see [`ch_width_for`]) or because no
shaper was in reach; `ch` then resolves through
[`LengthContext::ch_base`]'s `0.5em` substitute.

The percentage basis is left unknown: a bare `%` is resolved later by layout
against the containing block, and a `%` term inside `calc()` therefore does
not fold here (that value falls back to `auto`/`0`, still without an
unsupported-feature diagnostic).
