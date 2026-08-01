Resolves a caret byte `index` to its painted geometry against the wrapped
`layout` (spec §5.10/§5.11): `(line_top, line_height, line_x_offset, x)`,
where `line_top` is the caret's line box top measured **down from the text
box origin** (add `origin.y`), `line_x_offset` is that line's alignment
offset, and `x` is the caret pen x measured from the line's content start
(so the painted x is `origin.x + line_x_offset + x − scroll_left`).

The pen x is the line-relative [`line_caret_x`]: the sum
of THIS line's spaced/tabbed glyph advances before `index`, i.e. the same
advances (incl. any per-line hang / justification, CSS Text 3 §8) the
segments paint from — so a caret on line 2 lands at that line's own x, not
the summed width of both lines, AND a field with `letter-spacing` /
`word-spacing` / `tab` places the caret on the spaced glyph edge rather than
a spacing-blind re-shape. A single-line, default-spacing, left-aligned field
→ line 0, top 0, `x_offset` 0, and the same spacing-0 advance sum the flat
re-shape produced — byte-identical.
