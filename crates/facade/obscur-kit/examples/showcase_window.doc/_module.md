**Windowed capability showcase → Vello (GPU, interactive).**

The GPU sibling of `showcase_png.rs`: it renders the *very same* shared
[`showcase`] scene (grid + flex layout, gradients, borders, radius, shadows,
blur, group opacity, transforms, a `background-image` + an `<img>` primitive,
and real shaped text driving the [`Button`]/[`Checkbox`]/[`Input`] widgets) —
but through winit 0.30 + Vello/wgpu 29 into a genuine on-screen window, so the
GPU-class effects render at **full fidelity** instead of the CPU
approximation. Concretely, on this backend:

- `linear-gradient` / `radial-gradient` / `conic-gradient` paint real smooth
  ramps (the CPU PNG shows only the first stop);
- `box-shadow` and `filter: drop-shadow(...)` paint real Gaussian-blurred
  shadows (the CPU PNG paints a flat offset rect);
- `opacity: 0.5` composites a real translucent group layer;
- `transform: rotate()/scale()`, borders, radius, images, and shaped text all
  render as on the CPU, but presented live.

It is also **interactive**, driving the shared [`ShowcaseState`] through the
runtime's real domain input + command path and rebuilding the view reactively:

- clicking the **Button** ("Click me") fires its `EVENT_CLICK` command, whose
  handler bumps `state.counter` — the toolbar's `counter = N` text updates;
- clicking the **Checkbox** fires its change command, toggling `state.checked`;
- the **Input** is editable: clicking it seats the caret, and typing / IME
  flows winit → converters → `UiRuntime` → the runtime-owned editing session,
  whose per-keystroke `on_text_input` handler mirrors the resulting text into
  `state.input_text` so the reactive builder repaints the field.

# Device scale, wheel scroll & zoom

The window is **device-scale aware**: the render scale is the display's HiDPI
`scale_factor` — **not tied to the window size**. It points the runtime's
viewport at `physical_extent / effective_scale` and sets
`UiRuntime::set_device_scale(effective_scale)` — the runtime then brackets the
whole scene in a root `scale(s, s)` transform, so vectors, text, and gradients
rasterize crisply at their authored CSS size. Because the scale is fixed to the
display density, **resizing the window reflows the layout** (the `1fr` gallery
tracks divide the real width; the UI is *not* zoomed up and down with the
window) — exactly like a real application. Any content taller than the window
is reached by the **mouse wheel** (`scroll_y` → `UiRuntime::set_content_offset`,
a root `translate`); when the content fits, scroll clamps to `0`. A user
**zoom** multiplies on top: `Ctrl+=` / `Ctrl++` zoom in (cap 500%), `Ctrl+-`
zooms out (min 30%), and `Ctrl+0` resets zoom **and** scroll. The initial window
is 1000×700 *logical* pixels.

Run it (a display + a GPU adapter are required):
`cargo run -p obscur-kit --example showcase_window --features full`

# Wiring

This is a thin driver over the shared scene and the existing
runtime/converter/Vello APIs — it adds no rendering logic of its own. The
window/device/surface/present path mirrors `windowed_counter.rs`; the scene,
stylesheet, and image-asset setup mirror `showcase_png.rs`. The image asset is
staged into a sandbox temp dir the runtime's filesystem loader is rooted at
(via [`KitBootstrap::install_image_resources`]), so both the
`background-image: url("gallery.png")` card and the `<img>` primitive resolve
to real decoded pixels. Text is a host resource: a system font (plus a
best-effort CJK fallback) is loaded off disk and wired as the full itemize →
resolve-by-coverage → shape stack, with the same id↔bytes assignment handed to
the Vello renderer.

# Environment note

Like `windowed_counter.rs`, this example needs a display and a usable GPU
adapter; in a headless environment it compiles but exits cleanly on
`resumed` when no adapter/surface can be acquired (it never panics on that
path). On a real machine it opens a window and presents the showcase live.
