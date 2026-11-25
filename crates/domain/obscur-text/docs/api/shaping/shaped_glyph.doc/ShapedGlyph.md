One shaped output glyph: a font-specific glyph index paired with its pen
advance, placement offsets, and the source byte it maps back to (spec §5.10).

Shaping is not a one-to-one mapping from characters to glyphs. The
[`cluster`](Self::cluster) field records the **byte offset** into the source
text this glyph originates from, which is what caret hit-testing walks back
through:

- A ligature collapses several source characters into one glyph, so multiple
  input bytes fold onto a single glyph and several adjacent glyphs may share
  the same cluster value.
- Conversely one character can decompose into multiple glyphs (e.g. a base
  plus combining marks), so several glyphs can carry the same cluster.

Advances and offsets are in logical pixels (already scaled for the requested
font size), keeping this type adapter-neutral. Storing `f32` fields means the
type derives `PartialEq` but not `Eq`.
