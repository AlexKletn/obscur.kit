Software/headless [`Renderer`](obscur_render::Renderer) adapter (spec §5.2
renderer abstraction, §2.7 "Software/headless renderer").

[`CpuRenderer`](renderer::CpuRenderer) implements the domain `Renderer` port
by walking a [`SceneSnapshot`](obscur_scene::scene::SceneSnapshot)'s display
list and rasterizing it into a CPU pixel buffer with `tiny-skia`. It holds no
GPU or OS handle, so it is the backend a headless golden-image test drives.

Per the determinism boundary of spec §5.2, this backend reproduces layout,
paint order, clipping, and the Scene/Paint snapshots identically to every
other backend; only rasterization, anti-aliasing, and precision may differ.
Effects it cannot reproduce surface as explicit diagnostics, never a panic.
