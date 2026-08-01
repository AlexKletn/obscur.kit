**Proof: CSS-authored motion actually moves.**

A `transition` and an `@keyframes` animation, authored in plain CSS, compiled
through the real CSS frontend, and driven by the runtime's motion scheduler —
rasterized through the software [`CpuRenderer`] as a five-panel filmstrip in
`target/css_motion_proof.png`:

- **The blue `.bar`** carries `transition: width 1s linear`. Hovering it
  changes its computed width `60px -> 300px`, so the transition fires and the
  bar *grows across the strip* instead of snapping.
- **The red `.ghost`** plays `@keyframes fade` via the `animation` shorthand,
  so it fades in `opacity: 0.15 -> 1` over the same second.

Each panel is one motion frame at `t = 0, 0.25, 0.5, 0.75, 1.0`. The bar's
width is read back from the runtime's own layout boxes and **asserted** at
every step, so a regression fails the run rather than printing a still image.

The wiring is the kit's, not this example's: [`CssPipeline::install`] hands the
sources to the CSS frontend and installs *both* halves of the result on the
runtime — the matchable rules and the `transition`/`@keyframes` motion (§4.17).
A host that installs only the rules gets a motionless UI, which is exactly why
the plumbing lives in the kit.

Run it:
`cargo run -p obscur-kit --example css_motion_proof --features render-cpu`
