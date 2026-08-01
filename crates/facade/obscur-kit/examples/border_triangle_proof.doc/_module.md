**Proof: the CSS border-triangle trick renders (headless CPU).**

The classic trick — a `width:0;height:0` box with three transparent border
sides and one colored side — draws a solid TRIANGLE (an arrow/marker glyph:
select ▼, disclosure ▸▾, stepper ▲▼). It works because the scene coordinator
now paints each SOLID border side as a **mitered trapezoid quad** (outer edge
full, inner edge inset by the two adjacent border widths) instead of an
axis-aligned rect: when a box collapses, the colored side's trapezoid folds
into a triangle, and transparent sides paint nothing.

This example mirrors the coordinator's `border_fills` geometry to emit the
four cardinal arrows (▽ ▲ ▷ ◁) plus a normal 4-colour bordered box (whose
corners are now 45°-mitered), rasterizes them through the software
[`CpuRenderer`] (no GPU), verifies each triangle is filled at its centroid yet
empty at an outside corner, and writes `target/border_triangle_proof.png`.

Run it:
`cargo run -p obscur-kit --example border_triangle_proof --features render-cpu`
