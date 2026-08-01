The byte a wrapped line's content **starts** at (spec §5.10). For a line
with glyphs this is its first glyph's cluster (`range().start`); for an
**empty** line box — a trailing/leading/consecutive forced break (`\n`)
leaves one, whose [`ParagraphLine`] defaults its start to `0` — the true
start is the *previous* line's end (or `0` for a leading empty line), so a
caret on the empty line after a final `\n` resolves to just past it.
