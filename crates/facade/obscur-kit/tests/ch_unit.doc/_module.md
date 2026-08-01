End-to-end: the **`ch` unit** resolves from the `0` glyph actually shaped by
the runtime's font, not from a stand-in (spec §4.5).

`ch` is the advance of `0` (ZERO, U+0030) in the element's own font — the unit
that sizes a box by character count (`width: 60ch`). CSS Values 4 §6.1.2 allows
`0.5em` as a substitute *only* "in the cases where it is impossible or
impractical to determine the measure of the ZERO glyph". That escape hatch is
the thing these tests exist to rule out: resolving every `ch` to `0.5em` would
look like support while mis-sizing the one case anyone writes `ch` for.

The default [`MockTextShaper`] is monospace at `0.6 × font-size` per glyph.
That ratio is what makes these assertions meaningful: a real measurement lands
on `0.6em` per `ch`, the substitute would land on `0.5em`, and every test below
is written so those two answers disagree. A `ch` that quietly fell back would
fail here rather than pass with plausible numbers.
