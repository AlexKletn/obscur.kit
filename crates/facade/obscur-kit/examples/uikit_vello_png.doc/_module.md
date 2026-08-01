**Headless imported-UI-Kit → PNG (faithful GPU render via Vello).**

The GPU sibling of `uikit_png.rs`: it renders the *very same* shared [`uikit`]
scene and the *very same* per-theme stylesheet, but through **Vello/wgpu** into
an offscreen texture that is then **read back** to CPU pixels and encoded to a
PNG — with no window on screen. This is the *faithful* render: the GPU-class
effects the CPU backend only approximates paint for real here.

Concretely, for the gradient/shadow-heavy **Skeuomorphic** theme (the default,
index 4):

- every `linear-gradient(#light,#dark)` face paints a real smooth vertical ramp
  (the CPU PNG shows only the first, light, stop — a flat fill);
- every `box-shadow` — the `inset 0 1px 0 rgba(255,255,255,…)` gloss highlights
  AND the `0 2px 5px rgba(0,0,0,…)` cast shadows, combined in one comma list —
  paints a real Gaussian-blurred shadow (the CPU PNG paints a flat offset rect);
- bevel borders, radius, and shaped text render as on the CPU, but at full GPU
  fidelity.

Select the theme with the `UIKIT_THEME` env var (0-based index into
[`uikit::themes`], default 4 = Skeuomorphic); e.g. `UIKIT_THEME=1` renders
Material for comparison. The output is `target/uikit_<slug>_vello.png`.

# Wiring

A thin driver over the shared scene and the existing runtime + Vello APIs: it
adds no rendering logic of its own. It acquires a **headless** wgpu device (no
surface — `request_adapter` with `compatible_surface: None`), builds a
[`VelloDevice`] + [`VelloRenderer`], wires the same host text stack as
`uikit_png.rs`, runs one frame through [`UiRuntime`], and calls the adapter's
[`VelloRenderer::render_to_image`] read-back helper to obtain `(w, h, rgba8)`,
which is encoded with `tiny_skia` (the bytes are premultiplied RGBA8, exactly
what `Pixmap::from_vec` expects).

# Environment note

Needs a usable GPU adapter (a display is available here). In a truly headless
environment with no adapter it prints a diagnostic and exits cleanly (no panic);
it never fabricates a device.

Run it (Skeuomorphic):
`cargo run -p obscur-kit --example uikit_vello_png \
  --features "render-vello resource-fs resource-decode text-harfrust text-unicode"`
