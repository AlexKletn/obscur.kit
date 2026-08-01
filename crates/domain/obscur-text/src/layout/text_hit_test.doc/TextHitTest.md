The resolved caret position produced by hitting a [`TextLayout`] with a point
(spec §5.10 caret).

It names which [`line_index`](Self::line_index) the point fell on, the source
[`byte_offset`](Self::byte_offset) of the nearest cluster boundary, and whether
the point landed on the *trailing* half of a glyph
([`is_trailing`](Self::is_trailing)) — in which case the caret belongs *after*
that cluster. It carries only integer/boolean fields, so it derives `Eq` and
`Hash`.
