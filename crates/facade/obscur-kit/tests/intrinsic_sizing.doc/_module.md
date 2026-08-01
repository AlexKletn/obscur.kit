End-to-end: the **intrinsic sizing keywords** (`min-content` / `max-content` /
`fit-content`) are answered by measuring content through the live pipeline
(spec §4.2, CSS Sizing 3 §4.1).

These three cannot be resolved by arithmetic — the answer is a property of the
content, so the box is sized by measuring its children a second time under a
different offer. That makes them easy to fake and easy to get subtly wrong, so
every test here asserts a real arranged box against a number derived from the
text, not against itself:

* `max-content` — everything on one line;
* `min-content` — the widest word, with the rest wrapped under it;
* `fit-content` — the content when it fits, the container when it does not,
  and never below the widest word.

The default shaper is monospace at `0.6 × font-size` per glyph, so a run of
`n` characters at 16px is exactly `n * 9.6` wide. Every expectation below is
written from the actual string, which is what makes a wrong answer visible
rather than plausible.

# The consumers, not just the `width`

A keyword being answered on `width` says nothing about the other places CSS
allows one, and for a long time `width` was the only place that measured. So
this file also drives the **bounds** (`min-width: min-content` flooring a
narrower authored width, `max-width: max-content` capping a stretching box)
and **grid** (`grid-template-columns: min-content 1fr`, which used to abort
the whole declaration at CSS lowering and take the `1fr` with it). The grid
tests use an ordinary element as the grid item — not a bare text leaf —
because that is the case that has to work and the case that was hollow.

Three places still degrade a keyword to `auto`, and are deliberately not
covered here because covering them would be asserting a wrong answer:

* a flex or grid **container's own** `width` / `height` — `display: flex;
  width: min-content` fills its parent;
* a flex or grid container's own `min-`/`max-` bounds, which impose no clamp
  (only flow measures its bounds);
* a flex item's **cross**-axis size and cross bounds.

Each needs a min-content pass over a formatting context that has none — flow's
probe stacks children, which is not what a flex row or a grid does.
