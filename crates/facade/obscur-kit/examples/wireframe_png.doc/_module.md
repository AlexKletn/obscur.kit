**Headless Browser-Elements-Wireframe-Kit → PNG.**

Renders the shared [`wireframe`] scene — the masthead over the 15 native-control
group cards — through the **software** [`CpuRenderer`] (no display, no GPU) and
writes the frame to `target/wireframe_kit.png`, a real viewable artifact.

The scene is a grayscale, flat "spec sheet": every native HTML control drawn
across all of its states. [`wireframe::view`] builds it from semantic classes
and [`wireframe::stylesheet`] supplies the wireframe user-agent appearance.
Because the design is grayscale + flat (solid fills, 1.5px borders, small radii,
text, flex, grid, absolute position), the CPU backend reproduces it near-
faithfully; the handful of engine gaps are approximated + documented in the
scene module (search "SUBSTITUTE" there).

Supersample factor: `UIKIT_SCALE` (default 2.0, clamped 1.0..=4.0). The sheet is
laid out at the LOGICAL [`VIEWPORT`] while the CPU render target is `VIEWPORT ×
scale`; `set_device_scale(scale)` brackets the scene in a root `scale()` so the
physical buffer fills crisply. `UIKIT_SCALE=1` reproduces the logical 1:1 output.

Run it:
`cargo run -p obscur-kit --example wireframe_png \
  --features "render-cpu resource-fs resource-decode text-harfrust text-unicode"`
