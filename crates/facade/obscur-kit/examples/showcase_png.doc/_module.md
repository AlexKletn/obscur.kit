**Headless capability showcase → PNG.**

Renders the shared [`showcase`] scene (grid + flex layout, gradients, borders,
radius, shadows, opacity, transforms, a `background-image` and an `<img>`
primitive, real shaped text, and the [`Button`]/[`Checkbox`]/[`Input`]
widgets) through the **software** [`CpuRenderer`] — no display and no GPU — and
writes the rasterized frame to `target/showcase_cpu.png`, a real viewable
artifact.

GPU-class effects (gradients, shadows, blur) degrade to the documented CPU
approximation per the project's effect-split policy: a gradient paints its
first stop, a shadow paints a flat offset rect. Layout, text, images, borders,
radius, opacity, and transform geometry render faithfully. Sub-step 2's
windowed Vello example shows the effects at full fidelity from the *same*
[`showcase`] module.

Text is a host resource: this example loads a system font off disk (best
effort) and wires the full itemize → resolve-by-coverage → shape stack, then
hands the **same** font bytes to the CPU renderer so glyphs rasterize as real
outlines. With no system font present it still runs — text falls back to the
backend's box-coverage marks (never a panic).

Run it:
`cargo run -p obscur-kit --example showcase_png \
  --features "render-cpu resource-fs resource-decode text-harfrust text-unicode"`
