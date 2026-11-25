The byte on wrapped line `li` nearest an x of `target_x` (measured from the
text box origin), clamped to that line's byte span (spec §5.11).

The line's content starts at pen x `line.x_offset()` from the box origin, so
the target is turned into a within-line pen x by undoing the alignment offset
(`target_x − line.x_offset()`) and resolved over the line's OWN spaced/tabbed
glyph advances with the SAME midpoint walk
[`byte_at_line_local_x`] uses — the exact inverse
of the [`line_caret_x`] the caret paints with, so a click
at a spaced glyph edge (or on a `text-align: right`/`center` line) resolves
back to that glyph's byte. The result is clamped to `[line start, line end']`,
where `end'` excludes a trailing `\n` so a click past the line's end sits
before the newline (on this line), not at the start of the next.
