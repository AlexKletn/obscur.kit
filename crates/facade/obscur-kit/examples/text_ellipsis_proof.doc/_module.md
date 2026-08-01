**Proof: `text-overflow: ellipsis` end-to-end through the real runtime.**

Renders text through the REAL runtime + software [`CpuRenderer`] (no GPU),
stitched top-to-bottom into `target/text_ellipsis_proof.png`, proving the
already-built `text-overflow: ellipsis` path (CSS UI 3 §5.2): a box narrower
than its (`white-space: nowrap`) text keeps the largest fitting prefix and
appends the ellipsis glyph `…` (U+2026), never exceeding the box.

Blocks (all `font-size: 18px`, `width: 60px`, no padding so the content box is
exactly 60px):

- **A — `text-overflow: ellipsis`** on a long `nowrap` phrase: the one
  overflowing line is truncated to a prefix + the ellipsis glyph; its last
  painted glyph is the ellipsis and the run width ≤ 60px.
- **B — `text-overflow: clip`** (control) on the SAME phrase: the line is
  truncated to a prefix too, but the last glyph is a NORMAL glyph, not the
  ellipsis — proving A actually appended the ellipsis.

Before writing the PNG it machine-checks the composed display list against a
standalone `…` probe render (to learn the font's ellipsis glyph index, which
is `0x2026` under the reference mock and a real font index under HarfRust) and
against a wide `nowrap` render of the phrase (to prove the box really is
narrower than the text).

Run it:
`cargo run -p obscur-kit --example text_ellipsis_proof \
  --features "render-cpu text-harfrust text-unicode"`
