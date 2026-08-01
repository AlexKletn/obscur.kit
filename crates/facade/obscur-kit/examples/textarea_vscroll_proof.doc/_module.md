**Proof: a multi-line `<textarea>` scrolls VERTICALLY to keep the caret's line
in view** (CSS Overflow 3 §3.1 scroll-into-view + §scroll-container).

Drives a focused `<textarea>` (a fixed-height `overflow: auto` box) through the
real runtime editing path: it types more lines than fit, renders a CPU
`target/textarea_vscroll_proof.png`, and asserts:

- the field's overflow scroll container scrolled (`scroll_top > 0`);
- the painted caret's LINE, offset by the container's `translate(0, −scroll_top)`,
  lands INSIDE the field box (its y within `[box.top, box.bottom]`) — you can see
  the line you are typing on;
- the TOP lines are scrolled out above the box (their unscrolled run y, minus
  `scroll_top`, sits above `box.top`) and clipped;
- a scrollbar thumb (an 8-px-wide fill at the box's right edge) paints.

Run it:
`cargo run -p obscur-kit --example textarea_vscroll_proof \
  --features "render-cpu text-harfrust text-unicode"`
