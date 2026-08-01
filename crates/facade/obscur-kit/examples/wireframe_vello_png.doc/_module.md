**Headless Browser-Elements-Wireframe-Kit → PNG (faithful GPU render via Vello).**

The GPU sibling of `wireframe_png.rs`: it renders the *very same* shared
[`wireframe`] scene and the *very same* wireframe user-agent stylesheet, but
through **Vello/wgpu** into an offscreen texture that is then **read back** to
CPU pixels and encoded to a PNG — with no window on screen. This is the
*faithful* render: the GPU-class effects the CPU backend only approximates paint
for real here.

Concretely, the sheet's `repeating-linear-gradient()` layers paint as REAL tiled
stripes on the GPU (the CPU PNG shows only the flat first stop, by policy):

- the **graph-paper page grid** — two crossing `repeating-linear-gradient()`s in
  one `background-image` comma list — draws its real 24px `#e2e0d9` grid;
- the **indeterminate progress** hatch and the **meter** stripe fills draw their
  real 45°/90° stripes;
- the **colour-swatch** checker draws its real hatch.

Everything else (real dashed disabled borders, real `outline` focus rings, the
CSS border-triangle arrows, the wrapping textarea, the inset error ring, and the
`overflow: hidden` corner-clipping) renders identically on both backends.

The output is `target/wireframe_kit_vello.png`.

# Wiring

A thin driver over the shared scene and the existing runtime + Vello APIs (a
faithful copy of `uikit_vello_png.rs`'s harness, retargeted at the wireframe
scene). It acquires a **headless** wgpu device (no surface), builds a
[`VelloDevice`] + [`VelloRenderer`], wires the same host text stack as
`wireframe_png.rs`, runs one frame through [`UiRuntime`], and calls
[`VelloRenderer::render_to_image`] to obtain `(w, h, rgba8)`, encoded with
`tiny_skia`.

# Environment note

Needs a usable GPU adapter. In a truly headless environment with no adapter it
prints a diagnostic and exits cleanly (no panic); it never fabricates a device.
The CPU approximation is always available via `wireframe_png`.

Run it:
`cargo run -p obscur-kit --example wireframe_vello_png \
  --features "render-vello resource-fs resource-decode text-harfrust text-unicode"`
