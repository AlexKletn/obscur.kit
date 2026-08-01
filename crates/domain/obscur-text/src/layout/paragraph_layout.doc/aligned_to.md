Returns this paragraph with every line re-aligned inside `width`.

Wrapping and alignment answer to **different widths**, and conflating them
is why `text-align` used to do nothing. A text box is shrink-wrapped to its
longest line, so aligning inside *it* can only ever produce a zero offset —
there is no slack to distribute. What CSS aligns within is the containing
block, which is wider. So the paragraph wraps to the width it was measured
against and is then re-aligned inside the width it is actually laid out in.

A no-op for `start`/`left` in LTR (offset `0` either way), which is what
keeps every existing paint byte-identical.
