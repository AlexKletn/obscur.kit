**Windowed capabilities demo → Vello (GPU, animated).**

The live sibling of `capabilities_png.rs` / `capabilities_vello_png.rs`: it
renders the *very same* shared [`capabilities`] scene through winit 0.30 +
Vello/wgpu 29 into a genuine on-screen window, so the GPU-class effects (the
Oklab gradient, the linearRGB blur, the spread shadow) render at **full
fidelity** live — and it additionally **animates** the transform-interpolation
card (§13).

# The animated transform

The CSS→motion plumbing is scalar-only today, so the interpolation is driven
at the DRIVER level (a clean demo-driver animation, no engine change): each
frame the driver blends `rotate(0°) → rotate(90°)` through the motion engine's
[`MotionValue::Transform`] decompose→lerp→recompose at an oscillating
`t ∈ [0, 1]`, and applies the resulting `[a, b, c, d, e, f]` affine to the
`.xform-box` element by layering a per-frame `.xform-box { transform:
matrix(...) }` override stylesheet on top of the base sheet (the runtime's
real CSS cascade path — a later same-specificity rule wins). The decomposition
keeps the square at FULL size through the whole sweep (a naive component-wise
matrix lerp would shrink it ~30% at the midpoint); the box rotates 0°→90°→0°.

# Device scale & HiDPI

The window is **device-scale aware**: the render scale is the display's HiDPI
`scale_factor` — **not tied to the window size**. It points the runtime's
viewport at `physical_extent / scale` and sets the runtime device scale so the
scene is bracketed in a root `scale()` and rasterizes crisply at its authored
CSS size. Because the scale is fixed to the display density, **resizing the
window reflows the layout** — the UI is *not* zoomed up and down with the
window — exactly like a real application.

Run it (a display + a GPU adapter are required):
`cargo run -p obscur-kit --example capabilities_window --features full`

# Environment note

Like the other windowed examples, this needs a display and a usable GPU
adapter; in a headless environment it compiles but exits cleanly on `resumed`
when no adapter/surface can be acquired (it never panics on that path).
