**Proof: `border-style: dashed` renders as REAL dashes (headless CPU).**

Builds a display list of two boxes with a `2px dashed #2b2a27` border — one
square, one rounded — exactly as the scene coordinator emits them: a single
`DrawPath` that STROKES the border-box outline (inset by `width/2`) with a
`Stroke::with_dash([k, k], 0.0)` pattern. It rasterizes through the software
[`CpuRenderer`] (no GPU) and writes `target/dashed_border_proof.png`.

Before writing, it scans the top border scanline of the square box and asserts
the stroke actually alternates painted dashes and empty gaps (not a solid
line), printing the run summary — a machine check that the dash took effect.

Run it:
`cargo run -p obscur-kit --example dashed_border_proof --features render-cpu`
