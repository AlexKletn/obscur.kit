**Headless capabilities demo → PNG (faithful GPU render via Vello).**

The GPU sibling of `capabilities_png.rs`: it renders the *very same* shared
[`capabilities`] scene through **Vello/wgpu** into an offscreen texture that
is read back to CPU pixels and encoded to `target/capabilities_vello.png` —
with no window on screen. This is the *faithful* render: the GPU-class effects
the CPU backend only approximates paint for real here.

Concretely:

- the **Oklab gradient** card (§2) paints a real `linear-gradient(90deg,
  #f00, #0f0)` interpolated in Oklab — a vivid yellow-green midpoint, not the
  muddy sRGB blend and not the CPU's flat first stop;
- the **linearRGB blur** card (§3) paints a real Gaussian blur convolved in
  linear light (the CPU PNG shows only a coarse approximation);
- everything else (layout, color parsing, radius/spread, stacking, flex/grid,
  margins, text, the interpolated `rotate(45°)` square) renders as on the CPU,
  but at full GPU fidelity.

# Environment note

Needs a usable GPU adapter (a display is available here). In a truly headless
environment with no adapter it prints a diagnostic and exits cleanly (no
panic); it never fabricates a device.

Run it:
`cargo run -p obscur-kit --example capabilities_vello_png \
  --features "render-vello text-harfrust text-unicode"`
