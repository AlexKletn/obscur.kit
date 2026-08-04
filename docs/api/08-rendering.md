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

## Canvas: drawing pixels somebody else produced

A canvas painter draws with the ordinary Paint model (spec §5.3), and one thing
it can draw is a buffer of foreign pixels — a video frame, a camera image, a
screen capture, the output of a software rasterizer.

```rust
runtime.register_canvas(CAMERA, Box::new(move |region, _paint| {
    let frame = source.latest();           // whatever your producer hands over
    let buffer = ExternalBuffer::new(
        frame.bytes(),
        frame.width(), frame.height(),
        frame.stride(),                    // BYTES per row, not width * 4
        BufferFormat::Bgra8,
        AlphaMode::Straight,
    ).expect("the description holds");
    region.draw_buffer(&buffer, BufferFit::Cover);
    true                                   // "still moving" — ask me again
}));
```

Two things fail *silently* rather than loudly, and `ExternalBuffer` exists to
close both:

* **Channel order.** `Bgra8` is what Windows, X11, Wayland and most of ffmpeg's
  packed output hand over. Reading it as RGBA draws a blue face.
* **Row stride.** Nearly every real producer pads rows to an alignment. Reading
  a padded buffer as a tight one shears the picture — every row slides further
  sideways than the last.

`Rgbx8` / `Bgrx8` / `Rgb8` / `Bgr8` / `Gray8` are also accepted, all opaque; the
`x` byte is padding and is deliberately **not** read as alpha.

`BufferFit` is `Stretch` (default), `Contain`, `Cover` or `Natural`, all centred;
the region clips whatever overflows. The painter's return value is "still
animating" — a live source returns `true` and gets a frame per tick.

**What it costs.** Counting copies of the pixels for one foreign frame:

| | |
| --- | --- |
| `ExternalBuffer::new` | **0** — it borrows, and only checks the description |
| `to_scene_image()` | **1** — the repack and the channel swizzle |
| publishing it as a `SceneImage` | **0** — the buffer is moved, not converted |
| display list, snapshot, scene clones | **0** — a `SceneImage` holds an `Arc<[u8]>` |
| Vello | **0** — the `Arc` is wrapped in a `Blob`, a refcount bump; then the GPU upload |
| CPU, premultiplied | **0** — the bytes are borrowed as a `PixmapRef` |
| CPU, straight alpha | **1** — premultiplying has to write somewhere |

So the repack is the whole price. Measured on a 1080p frame:

| | ms/frame |
| --- | --- |
| repack, BGRA with padded rows | **2.4** |
| repack, already tight RGBA8 | **0.9** (one `memcpy` — the floor) |
| the CPU blit itself, for scale | ~13 |

A producer that can already hand over tightly packed **premultiplied** RGBA8 in
an `Arc<[u8]>` should skip `ExternalBuffer` entirely, build a `SceneImage`
directly and use `draw_image_into` — that path copies the pixels **not once**,
on either backend.

Two things about those numbers are worth knowing, because both were surprises:

* `SceneImage` holds `SharedPixels`, which is either an `Arc<[u8]>` or a moved
  `Vec<u8>`. That second shape is not a convenience — `Arc<[u8]>::from(vec)`
  allocates a *second* buffer and copies into it, which measured **6 ms a
  frame** at 1080p, more than the repack it was publishing.
* Building the repacked buffer by pushing pixels onto a `Vec` cost **13 ms**
  against the 0.9 the same bytes cost as a `memcpy`. It is allocated once at
  its final size and written into.

> **unbuilt** — spec §5.5 also allows a *texture-backed* foreign surface handed
> to the GPU without ever touching the CPU, and a native child surface/window.
> `ForeignSurface` in `obscur-scene` describes the policy (kind, limits) but
> carries no content and paints nothing. This CPU-buffer road is the one that
> works.

## Caches

> **unwired** — the three Vello caches (`glyph_cache`, `path_cache`,
> `texture_cache`) have no production caller; Vello's own atlases make them
> redundant. They are scaffolding to delete, not a tier awaiting a consumer.
> The decoded-image cache on the runtime *is* live and bounded.
