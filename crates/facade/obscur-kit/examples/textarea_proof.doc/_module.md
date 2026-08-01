**Proof: a multi-line `<textarea>` edits, wraps, and paints a LINE-AWARE caret**
(spec §5.10/§5.11; CSS Text 3 §4.1.1 segment breaks).

Drives a focused **multi-line** field (`type=textarea`, `white-space: pre-wrap`)
through the real runtime editing path: it types `"line one\nline two\nline
three"` — the two `\n`s entered as real **Enter** key presses, which the editing
keymap inserts as segment breaks (a single-line `input` would submit instead) —
renders a CPU `target/textarea_proof.png`, and asserts:

- the value laid out onto **three** lines (one `DrawText` run per wrapped line),
  each exactly `line-height` (22px) tall;
- the caret (at the end of the text) sits on the **third** line — its painted
  `y ≈ 2 × line-height` — at an `x` measured **within line 3** (the width of
  `"line three"` alone), NOT the summed width of all three lines (the single-line
  caret bug this replaces).

Run it:
`cargo run -p obscur-kit --example textarea_proof \
  --features "render-cpu text-harfrust text-unicode"`
