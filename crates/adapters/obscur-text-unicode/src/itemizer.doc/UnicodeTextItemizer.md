A real [`TextItemizer`] that splits text by bidi level and script (spec §5.10).

Itemization runs in two nested passes over the source text in logical order:

1. **Bidi.** `unicode-bidi` resolves the Unicode Bidirectional Algorithm for
   the whole string, seeded by the caller's base direction, producing one
   embedding [`Level`] per byte. A run is cut wherever the level changes; the
   run's [`direction`](TextItem::direction) follows the level parity (even =
   left-to-right, odd = right-to-left) and its [`level`](TextItem::level) is
   that embedding level.
2. **Script.** Within a single-level run the text is cut again wherever the
   `unicode-script` script changes. Script-neutral characters (`Common` /
   `Inherited` — spaces, punctuation, digits, combining marks) never start a
   new run: they extend the current one, and a leading neutral prefix adopts
   the script of the run that follows it.

The emitted items are contiguous, non-overlapping, and cover `0..text.len()`
with no gaps, so every range lands on a UTF-8 character boundary. An empty
string yields a single empty item `0..0` carrying the base direction (level
`0` for LTR, `1` for RTL) and script [`ScriptTag::COMMON`], matching the
reference `SingleRunItemizer`.
