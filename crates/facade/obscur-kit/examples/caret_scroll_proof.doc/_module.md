**Proof: a single-line text field scrolls horizontally to keep the caret in
view** (CSS Overflow 3 §3.1 scroll-into-view; the standard text-control
behaviour a `<input type=text>` scrolls with).

Drives a focused single-line `field` (100px wide, `white-space: nowrap`,
`overflow: hidden`) through the real runtime editing path: it types a value
far WIDER than the box, renders a CPU `target/caret_scroll_proof.png`, and
asserts:

- the painted caret sits INSIDE the field box (`caret.x ≤ box.right`) — you can
  see what you type;
- the text run's left edge is scrolled OUT past the box's left (head clipped),
  so the TAIL (last-typed chars) is what shows;
- pressing **Home** resets the scroll so the HEAD is visible again (run left
  back at the box's left edge).

Two frames are rendered side by side into one PNG: the scrolled "tail" state
and the reset "head" state, each with the caret drawn.

Run it:
`cargo run -p obscur-kit --example caret_scroll_proof \
  --features "render-cpu text-harfrust text-unicode"`
