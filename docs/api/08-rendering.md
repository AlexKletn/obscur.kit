# 08 — Rendering

The runtime produces a `SceneSnapshot`: an immutable, backend-independent
display list. A renderer consumes it. Nothing in the runtime knows which
renderer, which is why the same frame goes to a GPU window, a PNG and a test.

```rust
let result   = rt.render_now(&view, &sheets);
let snapshot = result.snapshot().clone();
let frame    = RenderFrame::new(snapshot, physical_size);
let submission = FrameSubmission::new(frame);
```

---

## Two renderers

### `CpuRenderer` — `obscur_kit::render_cpu`

Software, `tiny-skia`, always available (`render-cpu` is in the default feature
set). No display, no GPU.

```rust
pub fn new(id: RendererId) -> Self
pub fn register_font(&mut self, id: u32, font_bytes: Vec<u8>)
pub fn set_diagnostic_sink(&mut self, sink: Arc<dyn DiagnosticSink>)
pub fn pixmap(&self) -> Option<&tiny_skia::Pixmap>
```

Render into a bitmap target, then read the pixmap:

```rust
let target = RenderTarget::bitmap(BitmapTarget::new(viewport));
renderer.render(&submission, &target)?;                 // the `Renderer` trait
let png = renderer.pixmap().unwrap().encode_png()?;
```

### `VelloRenderer` — `obscur_kit::render_vello`

GPU, through `vello` + `wgpu`. Behind the `render-vello` feature.

```rust
pub fn new(id: RendererId) -> Self
pub fn with_device(self, device: VelloDevice) -> Self
pub fn set_device(&mut self, device: VelloDevice)
pub fn register_font(&mut self, id: u32, font_bytes: Vec<u8>)
pub fn set_font(&mut self, font_bytes: Vec<u8>)          // shorthand for handle 0
pub fn present_to_surface(&mut self, submission: &FrameSubmission,
                          surface: &VelloSurface) -> Result<PresentResult, RendererError>
pub fn render_to_image(&mut self, submission: &FrameSubmission,
                       width: u32, height: u32) -> Result<(u32, u32, Vec<u8>), RendererError>
```

`VelloDevice::new(wgpu::Device, wgpu::Queue)` wraps an acquired device;
`VelloSurface` handles `configure`, `resize` and `current_texture`.

`render_to_image` renders offscreen and reads back premultiplied RGBA8 — a
faithful GPU render with no window, which is what the `*_vello_png` examples use.

## Which one, and what differs

The CPU backend is not a second implementation of the GPU one; it **approximates
GPU-class effects by policy**. Gradients paint their first stop as a flat fill;
`box-shadow` paints a flat offset rect rather than a Gaussian; `filter: blur()`
is not blurred. Geometry, text, radii, clipping and borders match.

That is deliberate, and stated rather than hidden: a CPU PNG is a correct
*layout and geometry* oracle, not a pixel-accurate preview.

## Fonts must be registered on **both** sides

A glyph run in the display list carries a font **handle** (a `u32`), not bytes.
The runtime's font resolver assigns handles; the renderer rasterizes from its own
handle→bytes table. If the two disagree, runs shaped against one face rasterize
against another and you get garbage glyphs — or, if the handle is missing
entirely, nothing at all.

```rust
let installed = boot.install_text_stack(&mut rt, &font_set);
for (handle, bytes) in installed.iter() {
    renderer.register_font(handle, bytes.to_vec());
}
```

And again for faces discovered mid-run — see
[07 — Text and fonts](07-text-and-fonts.md#fallback-arrives-mid-frame).

## Targets

`RenderTarget` is `Window(WindowTarget)` · `Texture(TextureTarget)` ·
`Bitmap(BitmapTarget)`.

## Logical versus physical

The runtime lays out in **logical (CSS) pixels**; a surface is **physical**.
`UiRuntime::set_surface(physical, scale)` sets both at once and derives the
logical viewport, which is the only place the two should be related.

This matters more than it sounds. On a 2× display a 280×500-pixel window is a
*140×250 CSS-pixel* viewport — fixed-pixel padding and radii are then twice as
large relative to every box, which is exactly how a keypad turns into a row of
empty pills. If a layout looks wrong only on one machine, check the scale first.

## Frame statistics

`OBSCUR_FRAME_STATS=1` prints, every 60 presented frames: mean build, snapshot
and present times, the implied fps if saturated, and the worst single frame.

`OBSCUR_NO_VSYNC=1` unpaces the swapchain, so a frame that is merely *paced*
stops looking like a frame that is slow.

These exist because `PerformanceSpan` carries no duration — see
[11 — Diagnostics](11-diagnostics.md).

## Transparency

The window is always created transparent with a premultiplied composite mode, so
**the bottom-most fill decides what the compositor sees**. A root with
`background-color: rgba(…, 0.62)` lets a blurring compositor read the wallpaper
through the whole app. There is deliberately no API for it — a flag would be a
second source of truth about the same pixel.

## Caches

> **unwired** — the three Vello caches (`glyph_cache`, `path_cache`,
> `texture_cache`) have no production caller; Vello's own atlases make them
> redundant. They are scaffolding to delete, not a tier awaiting a consumer.
> The decoded-image cache on the runtime *is* live and bounded.
