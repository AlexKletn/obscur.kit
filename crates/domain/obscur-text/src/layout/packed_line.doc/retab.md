Re-expands every preserved tab (U+0009) so it advances to the next
`tab_width` stop measured from the **start of its own line** (CSS Text 3
§4.3), rewriting the tab glyphs' advances in place per packed line.

For each line the pen is reset to zero at the line's left edge and walked in
logical order; when a glyph is a tab its advance is set to `next − pen`,
where `next = (floor(pen / tab_width) + 1) · tab_width` is the next stop
strictly past the pen (§4.3: a tab always advances), and the recomputed
advance is folded into the pen so a following tab measures from it. Only tab
glyphs change — glyph id, cluster, offsets, font, level, and synthesis are
preserved. This is the exact per-line analogue of the pre-wrap
[`apply_tab_stops`](crate::shaping::apply_tab_stops) pen, which resets only
at hard `\n`; running it here — after wrapping knows the soft-wrap
boundaries — is what makes a tab on a wrapped line measure from that line's
origin. On line 0 the pen is identical to the pre-wrap pen, so the advance
is unchanged. Assumes `tab_width > 0`.
