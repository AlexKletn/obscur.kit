Resolves a run-local pen x (`local_x`, logical pixels measured from the
text run's left edge) to the nearest cluster byte offset in `content` —
the **inverse** of [`caret_offset_x`], used to place
the caret at a pointer click within the focused editable (spec §5.11).

`style` is the text node's computed style: its `font-family` /
`font-weight` / `font-style` / `direction` / `font-size` drive the fallback
chain, shaping attributes, base direction, and size — the **same** inputs
the compose pass shapes the drawn run with (and the same
[`caret_offset_x`] measures with), so a click at the
rendered caret x of offset `i` resolves back to `i`.

The text is shaped **once** through the same paragraph orchestrator (and
default face) the runs are drawn with, then the glyph advances are walked:
the boundary whose horizontal midpoint `local_x` falls at or before is
chosen (so a tie resolves to the earlier boundary). A `local_x` at or before
the run start clamps to `0`; a `local_x` past the last glyph clamps to the
content length. Single-line, left-to-right — the same documented
approximation as [`caret_offset_x`].
