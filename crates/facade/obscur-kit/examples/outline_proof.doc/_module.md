**Proof: `outline` + `outline-offset` paints a focus ring OUTSIDE the box.**

Builds a display list of three filled boxes, each with a `2px` outline offset
`3px` — exactly as the scene coordinator emits it: a single `DrawPath` that
STROKES a rect (or rounded rect) positioned at the border box INFLATED by
`outline-offset + width/2`, so the ring's inner edge sits `offset` px outside
the box and does not touch it. The three variants are a dashed outline
(`2px dashed #2b2a27`), a solid outline, and a rounded box with a rounded
outline. It rasterizes through the software [`CpuRenderer`] (no GPU) and writes
`target/outline_proof.png`.

Before writing it machine-checks the dashed box: (1) ink pixels exist OUTSIDE
the box's left edge — the ring is outside the box; (2) the column one pixel
left of the box is empty (white) — the `outline-offset` gap is visible; and
(3) the top ring edge alternates painted dashes and gaps — the outline is
dashed, not solid.

Run it:
`cargo run -p obscur-kit --example outline_proof --features render-cpu`
