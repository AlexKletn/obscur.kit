**Headless capabilities demo → PNG (software CPU).**

Renders the shared [`capabilities`] scene — every CSS feature ported in the
math campaign, authored as a CSS `&'static str` + a `View` tree — through the
software [`CpuRenderer`] (no display, no GPU) and writes the rasterized frame
to `target/capabilities_cpu.png`, a real viewable artifact.

This is the CPU-faithful subset: layout (flex + grid + block flow), CSS
Color 4 parsing (`oklch`/`lab`/`hsl`/`oklab` swatches), the `border-radius`
overlap clamp, `box-shadow` spread geometry, §9.9 stacking order, flex
grow/shrink distribution, grid two-value tracks, margin collapsing, the text
spacing/transform/`white-space`/ellipsis runs, and the interpolated
`rotate(45°)` square all render for real. The GPU-class effects — the Oklab
gradient (§2) and the linearRGB blur (§3) — degrade to the documented CPU
approximation (a gradient paints its first stop; a blur is coarse), so those
two cards carry a "GPU / Vello" caption here. `capabilities_vello_png.rs`
shows them for real.

Text is a host resource: the example loads a system font off disk (best
effort) and wires the full itemize → resolve-by-coverage → shape stack, then
hands the SAME font bytes to the CPU renderer so glyphs rasterize as real
outlines. With no system font present it still runs — text falls back to box
marks (never a panic).

Run it:
`cargo run -p obscur-kit --example capabilities_png \
  --features "render-cpu text-harfrust text-unicode"`
