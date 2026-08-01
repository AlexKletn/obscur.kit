Disambiguates the visual position of a byte offset that sits on a soft
line-wrap boundary (spec §5.11 editing engine).

When a paragraph is soft-wrapped, the byte offset at a wrap point is shared
by two visual positions: the *end* of the line above the break and the
*start* of the line below it. A single byte offset therefore cannot, on its
own, say where the caret should be drawn. `CaretAffinity` resolves the
ambiguity by naming which position the offset binds to:

- [`Upstream`](Self::Upstream) binds the offset to the *preceding* position:
  the trailing edge of the earlier line (end of line `N`).
- [`Downstream`](Self::Downstream) binds the offset to the *following*
  position: the leading edge of the later line (start of line `N + 1`).

[`Downstream`](Self::Downstream) is the [`Default`]: a freshly placed caret
binds forward, toward the start of the next line. Affinity only matters at
wrap boundaries; anywhere else the two directions describe the same visual
position.
