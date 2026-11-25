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

## Subpixel paint rectangles

`UiRuntime::snap_window_paint_rect(rect, PixelSnapMode::Edges)` aligns the
edges of an explicitly supplied paint rectangle at the current device scale.
Snap adjacent rectangles by their shared edge to avoid gaps from rounding
widths separately. `Auto` and `None` preserve the rectangle in this helper.
Layout and hit testing continue to use the original fractional geometry.

Set `pixel-snap: edges` to align a node's rectangular CSS background to
physical pixel edges. The surface scale, content offset, and ancestor scroll
offsets determine the grid. The property is local to the node. `auto` (the
default) and `none` preserve fractional background bounds. Rounded backgrounds
and backgrounds inside CSS transforms retain their original geometry.

## Renderers

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

### Experimental Sparse Strips renderer

Build the kit with `render-vello-hybrid` and set `OBSCUR_RENDERER=vello-hybrid`
to select the Sparse Strips renderer for `App::run` and `App::layer`.

```sh
OBSCUR_RENDERER=vello-hybrid cargo run -p obscur-kit \
  --features full,render-vello-hybrid --example hello_app
```

For a layer-shell surface, use `--example panel`. Embedders can attach an
existing wgpu device and queue with `VelloRenderer::with_hybrid_device`.
The prototype uses `vello_hybrid` 0.0.9 on the kit's wgpu 29. It handles paths,
gradients, clips, text, group opacity and outset shadows. Images, masks,
inset shadows, layer filters and backdrop blur are currently omitted.

### Lyon renderer

Build with `render-lyon` and select `OBSCUR_RENDERER=lyon` for an ordinary
window or a layer-shell surface.

```sh
OBSCUR_RENDERER=lyon cargo run -p obscur-kit \
  --features full,render-lyon --example calculator
```

Use `--example panel` to try the layer-shell path. Applications can also call
`App::with_renderer(RenderBackend::Lyon)`, which takes precedence over the
environment. The default is classic Vello. Set `OBSCUR_RENDERER=vello` to
select it explicitly. A missing feature or an unknown renderer name returns a
startup error.

Lyon tessellates solid paths, dashed and solid strokes, and glyph outlines, and uses multisampled
stencil clips for nested clipping. Normal opacity groups are composited in Lyon,
including nested groups and their clips. Unchanged frames reuse their uploaded mesh.
Paint changes can reuse tessellated contours. A frame that only changes transforms or the
opacity of a group containing one fill can reuse GPU vertices and indices
and update its draw parameters. The fast path supports up to 255 transform
and layer commands combined. Groups with overlapping draws still use the
full isolated composition path. Lyon supports every scene blend mode and
composition operator, including source replacement, destination over, and
clear. Non-default modes read the parent color per MSAA sample and reuse a
scratch target. Source replacement and clear affect transparent source pixels
inside the active clip. Hard shadows use tessellated contours. Soft inset and outset shadows use
a shader approximation to a blurred rounded rectangle. Inset shadows stay
inside the original rounded box and honor the signed spread and offset. Linear, elliptical radial, and conic
gradients support premultiplied Oklab interpolation and extend modes. Gradient
records must fit in a 16 KiB shader buffer and use local vertex coordinates.
RGBA images support straight and premultiplied alpha, clipping, transforms,
and independent tiling on either axis. Lyon retains textures used in
consecutive frames. Isolated groups support the CSS color filters compiled
into a color matrix. The Lyon matrix path preserves transparent pixels and
accepts up to 128 distinct combinations of matrix and group opacity per frame.
Foreground `filter: blur()` uses the shared separable Gaussian over a Lyon
group, after any color matrix. The compositor reuses scratch textures and
scissors the Gaussian passes to the padded layer bounds. Backdrop blur samples
the paint accumulated behind the panel and replaces it inside the rounded
panel and ancestor clips. Later foreground content stays sharp. Lyon draws
outline glyphs with synthetic bold, oblique, or both. Raster font strikes
use the Lyon texture cache and support PNG, premultiplied BGRA, and alpha
mask data. Lyon draws COLRv0 layers using the default CPAL palette and the
run brush for foreground entries. COLRv1 supports solid paints, palette and
foreground alpha, affine transforms, clipping, and all 28 composite modes.
COLRv1 gradients interpolate in premultiplied linear RGB. Multilayer glyphs
composite in an isolated linear RGB target and convert to sRGB before
joining the scene. `Clip::image_mask` supplies a raster mask for a standalone
clip or a layer clip. Lyon supports image alpha and linear-light luminance,
with zero coverage outside the image bounds. Classic Vello and the CPU
backend report mask coverage as unavailable. Lyon still falls back for
source-less masks and image masks on layers that use source replacement or
clear. Standalone applications report these fallbacks. Set
`OBSCUR_LYON_STRICT=1` when running a standalone application with Lyon to
return a render error instead of initializing classic Vello.

Embedders can attach the backend with `VelloRenderer::with_lyon_device` or
install `VelloDevice::lyon(device, queue).without_lyon_fallback()` for strict
mode.

`render_to_image` renders offscreen and reads back premultiplied RGBA8 — a
faithful GPU render with no window, which is what the `*_vello_png` examples use.

## Which one, and what differs

The CPU backend is not a second implementation of the GPU one; it **approximates
GPU-class effects by policy**. Gradients paint their first stop as a flat fill;
`box-shadow` paints a flat offset rect rather than a Gaussian; `filter: blur()`
is not blurred. Geometry, text, radii, clipping and borders match.

That is deliberate, and stated rather than hidden: a CPU PNG is a correct
*layout and geometry* oracle, not a pixel-accurate preview.

## Wayland background blur

Use `App::with_background_blur(true)` for a regular Wayland window or a layer-shell
surface. The request covers the entire surface. Use translucent backgrounds to
show the effect through the rendered content. Use
`app.control().set_background_blur(enabled)` to change the request while the
application is running. For ordinary Wayland windows, the kit derives a conservative
opaque rectangle from the rendered scene, reports it to the compositor and
excludes it from blur. Set `OBSCUR_WAYLAND_FULL_BLUR=1` for comparisons with the
whole window shape.

Kit prefers `ext-background-effect-v1` with the blur capability and uses legacy
KDE blur when available as a fallback. The compositor chooses the blur strength.
On other platforms or when neither protocol is available, the application opens
without compositor blur. This setting is independent of the renderer and CSS
`backdrop-filter`.

For regular windows, a diagnostic sink can observe
`window.background-blur.staged`, `window.background-blur.committed`, and
`window.background-blur.unavailable`. A commit records submission of the
requested state. Wayland provides no event confirming that the compositor
visually applied the effect.

On a regular window, a single root element that fills the viewport can set
`border-radius` to shape the blur. The region uses the same resolved corner
radii as painting, approximated by integer rectangles inside the outline.
The kit updates it on the next surface commit when the viewport or radii change.

For Adwaita, the Linux appearance provider exposes the ordinary window radius
as `--obscur-sys-window-radius` when GJS and libadwaita are available. It reads
the installed libadwaita stylesheet and supported direct radius overrides
in the user's GTK 4 CSS. Read appearance again to pick up edits to that file.
Use the variable in the root's `border-radius` to opt in.

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

## Switching renderers at runtime

Call `app.control().set_renderer(RenderBackend::Lyon)` from a UI command
handler to switch a running window or layer surface. The runner changes
the backend at a frame boundary and reuses the GPU device, swapchain, UI
state, and registered fonts. Requests before the frame coalesce. The same
Cargo feature requirements apply as for startup selection.

The runner emits `window.renderer.changed` with the initialization time.
An unavailable backend emits `window.renderer.unavailable` and leaves the
current renderer active. In ordinary windows, frame statistics start a new
sample after the switch. The first submitted frame is marked
`warmup=true` in the detailed log.

## Frame statistics

`OBSCUR_FRAME_STATS=1` prints, every 60 submitted frames: mean build, snapshot
and present times, the implied fps if saturated, and the worst single frame.

`OBSCUR_FRAME_STATS=each` adds per-frame timings in milliseconds. The runtime
logs reconcile/build, style, layout, and compose. The renderer logs raster
preparation and submission, the texture blit and its submission, and the
surface commit. The window runner also logs acquisition, snapshot, total
application work, and the interval between commits. Submission timings measure
the CPU side of GPU work.
The `[frame]` line also reports how many nodes resolved computed styles.
`[reconcile-work]` counts insertions, removals, moves, and updates across the
root view and nested components. Compare the counts with `styled` to see
whether an expensive frame chiefly creates nodes or restyles existing ones.
On frames that run layout, `[layout-work]` reports fresh measurements and
arrangements plus time spent preparing layout styles, solving layout, and
updating the hit-test index. Text measurement time is included in solve time.
Use `OBSCUR_FULL_LAYOUT_STYLE_MAP=1` to rebuild layout styles from computed
CSS on every layout pass for comparison. The default retains that mapping
and updates changed nodes during incremental layout. Set `OBSCUR_FULL_LAYOUT_TOPOLOGY=1`
to also rebuild the layout dependency index on every incremental pass.

On a native Wayland window with `wp_presentation`, the runner logs the
compositor's presentation or discard for each frame ID. It reports
`shown_fps` and `shown_frame_ms` over roughly one second of presentation
timestamps and publishes the same sample as a `window.performance`
diagnostic.

`OBSCUR_NO_VSYNC=1` unpaces the swapchain, so a frame that is merely *paced*
stops looking like a frame that is slow.

These exist because `PerformanceSpan` carries no duration — see
[11 — Diagnostics](11-diagnostics.md).

`OBSCUR_PREPARE_FRAME=1` enables experimental scene preparation in the
ordinary winit window. Only isolated runtime-owned animation ticks are
prepared before redraw. Input, resize, resource results, and application
requests supersede a prepared scene. Once a scene is ready, the runner
starts acquiring its swapchain image while waiting for redraw. Presentation
still follows the window adapter's redraw timing. The frame log marks accepted prepared
scenes with `prepared_ahead=true`.

### Animation and frame pacing probe

```sh
cargo build -p obscur-kit --release --example frame_probe \
  --features full,render-lyon,render-vello-hybrid
OBSCUR_FRAME_STATS=each OBSCUR_RENDERER=lyon \
  target/release/examples/frame_probe --transition
```

Run the same scene with `--translate` (transform), `--resize` (layout),
`--fade` (layer opacity), `--transition`, or `--idle`. The transition combines
a moving screen with real CSS width animations on an overflow clip and a
gradient shadow. The obscured screen keeps its fixed width. The modes use CSS
keyframes. Its buttons cycle modes and renderers and toggle compositor blur.
Use `--blur` to request blur at startup. `--idle --stress` repeatedly requests
frames for an unchanged scene to measure throughput without CSS animations.
During normal animation runs the toolkit schedules its own frames. Use fade
to measure opacity groups. Check fallback warnings when comparing renderers.

To measure rendering capacity separately from the display, run the same
tile animation offscreen:

```sh
OBSCUR_RENDERER=lyon OBSCUR_LYON_STRICT=1 OBSCUR_GPU_STATS=1 \
  target/release/examples/frame_probe --offscreen --transition --size=720x480
```

The probe advances the CSS timeline by a fixed virtual 1/120 second per
frame and reports completed frames per wall-clock second. It measures one
frame in flight and batches of eight, and reports preparation, CPU-side
submission and GPU completion wait separately. Offscreen mode omits the
window controls, compositor and swapchain. `--idle` reuses the retained
static scene and measures rendering it repeatedly. Add `--freeze` to repeat
the same snapshot of an animation after a short lead-in. With the resolved
cache enabled, a flat frozen scene isolates the cost of copying or blitting
the cached image. Fallback warnings
identify workloads that use a different backend internally.

Set `OBSCUR_GPU_STATS=1` to enable asynchronous Lyon GPU timestamps in either
mode. `[gpu-frame]` lines identify each Lyon submission within the current
device instance. `raster_ms` covers GPU execution of the Lyon encoder, including
its groups and resolve. It excludes CPU preparation, the swapchain blit, and
presentation. Drivers without encoder timestamps report this once and still
render normally. Samples may arrive on later submissions. Compare throughput
with the flag off because timestamp queries and per-frame logging add work.
Set `OBSCUR_GPU_GROUP_STATS=1` alongside `OBSCUR_GPU_STATS=1` to measure group
passes. `[gpu-lyon-groups]` reports root passes, passes into isolated groups,
and batches for blur and blend effects. Composites fused into a raster pass
are included in that pass's timing.
Set `OBSCUR_DAMAGE_STATS=1` to report the fraction of changed pixels and the
fraction of the frame covered by dirty 32×32-pixel tiles. `[damage-tiles]`
compares 16, 32, 64, and 128 physical pixels per tile side and counts dirty
tiles and horizontal runs of adjacent dirty tiles. These counts estimate
redraw area and fragmentation. `[damage]` compares fully rendered frames
and reports `gap` when samples are skipped. This
diagnostic uses an offscreen target for windows and reads pixels back, so
leave it disabled when benchmarking FPS or frame latency. In Frame Probe, use `--transition --clip-only` to keep the incoming page fixed
while the clip and shadow continue animating.

Set `OBSCUR_SCENE_DAMAGE_STATS=1` to report conservative Lyon present damage
from consecutive scene snapshots. Run it together with `OBSCUR_DAMAGE_STATS=1`
to check that each proposed region covers the actual changed pixels. These
diagnostics do not change the Wayland present request.

Set `OBSCUR_LYON_PARTIAL_REDRAW=resolved` to store a single-sample frame
cache and redraw merged dirty tiles using transient multisampled attachments.
The mode defaults to 32 physical pixels per tile. Set `OBSCUR_LYON_DAMAGE_TILE=16`
or `64` to compare. The resolved cache is copied or blitted into the output.
Set `OBSCUR_LYON_PARTIAL_REDRAW=msaa` to compare with the retained
multisampled attachment and full-frame resolve. Full redraw remains the default.
Frames with compositing
groups, unbounded changes, or excessive fragmentation still render in full.
`OBSCUR_GPU_STATS=1` adds `[lyon-partial]` decisions to the timing trace.

Set `OBSCUR_LYON_CROP_GROUPS=1` to try a smaller intermediate attachment for
a single ordinary opacity group. The group must have no ancestor stencil clip,
filters, or special blend mode. Clips inside the group are supported. The
attachment dimensions are rounded up to 32-pixel buckets and cropping is used
only when it saves at least a quarter of the target area.

Set `OBSCUR_LYON_SCHEDULE_GROUPS=1` to render ordinary opacity groups before
their parent and composite them in display-list order. This can reduce the root
to one pass. The renderer replays ancestor stencil clips, preserves 4× MSAA,
and uses one full-size color target per group. It uses the usual grouped path
for backdrop effects, filters, special blend modes, or scenes above the
experimental limit of six groups or 128 MiB of group color attachments.

Set `OBSCUR_LYON_CACHE_GROUPS=1` alongside scheduled groups to retain a group's
color attachment when its drawing inputs, ancestor clips, and nested composites
match. The group's own opacity is applied at composition time. Moving the
content or changing its paint, clip, or image invalidates the corresponding
cache entry. A child's content revision or opacity invalidates the parent
composite when the child is visible.
With GPU group tracing enabled, `[lyon-group-cache]` reports reused targets.
Set `OBSCUR_LYON_CROP_SCHEDULED_GROUPS=1` with scheduled groups and
deferred outer clips to give eligible leaf groups smaller color and stencil
attachments. The crop preserves physical-pixel alignment and 4× MSAA and
uses the group's paint bounds. GPU group tracing reports cropped passes as
`[lyon-scheduled-crop]`.

Set `OBSCUR_LYON_CLIP_SCISSOR=1` with deferred outer clips to intersect
scheduled groups' compositing rectangles with conservative clip bounds.
The stencil still applies the exact clip shape. GPU group tracing reports
both areas and the allocated texture area on `[lyon-group-scissor]` lines.

Set `OBSCUR_LYON_CACHE_ROOT=1` alongside scheduled groups and group caching
to cache root drawing commands that precede the first group. Lyon composites
that prefix per MSAA sample and replays any active stencil clips before drawing
the remaining commands. Changes to the prefix invalidate its cached texture.
`[lyon-root-cache]` reports cache hits with GPU group tracing enabled.

Set `OBSCUR_LYON_CACHE_TRANSLATIONS=1` to also reuse fully visible leaf groups
when they move by whole physical pixels. The group must have no ancestor
stencil clip and use integral transform translations. The compositor shifts the
texture lookup and reports that reuse as `shifted`.
Set `OBSCUR_LYON_CACHE_OUTER_CLIPS=1` to apply an ancestor stencil clip at
composition instead of rasterizing it into the leaf group. This is supported
when all ancestor clips remain active throughout the group. The same group
texture can then serve multiple clip sizes. Group tracing reports
`deferred_clip` and `clipped_hits`.
`[lyon-upload]` reports the bytes written and the full byte counts for vertices,
indices, and gradients. `retained=true` indicates that Lyon patched the existing
mesh on the CPU, such as when a rectangular clip or a fill with a dedicated
linear gradient resizes. Small updates across the three GPU buffers share one
staging write.

Lyon resolves MSAA directly into the window's RGBA8 or BGRA8 unorm view. The
view preserves encoded premultiplied color. Other surface formats and Vello
keep the intermediate render target and presentation blit.
Set `OBSCUR_LYON_SURFACE_BLIT=1` to compare the two paths.

On Wayland windows with presentation feedback,
`OBSCUR_WAYLAND_REDRAW_UNPACED=1` disables winit's frame-callback gate for
diagnostic comparisons. The swapchain and presentation feedback stay active.
This can produce more commits than the compositor presents and is intended
only for tracing frame pacing.
Set `OBSCUR_WAYLAND_REDRAW_AHEAD=1` to try timer-driven redraws at the monitor
refresh rate, with up to two frames waiting for presentation feedback. The
experiment returns to compositor callbacks if feedback stops. Set
`OBSCUR_WAYLAND_REDRAW_AHEAD=feedback` to keep the two-frame limit and
feedback timeout while disabling the refresh timer. Set
`OBSCUR_WAYLAND_REDRAW_AHEAD=display` to phase animation preparation to the
last Wayland presentation and monitor refresh period. This experiment uses
Linux monotonic presentation timestamps when available.
Set `OBSCUR_WAYLAND_MAX_PENDING=3` to compare a three-frame limit with the
default two. This caps outstanding presentation feedback. Swapchain buffering is
configured separately. Set `OBSCUR_WAYLAND_REDRAW_AHEAD=swapchain` to use
the monitor-period timer and swapchain readiness without waiting for presentation
feedback to admit another frame. Wayland still reports presented and discarded frames
when detailed frame statistics are enabled.

For frame pacing experiments, `OBSCUR_FRAME_LATENCY=1`, `2`, or `3` overrides
the wgpu surface's desired maximum frame latency. The backend may clamp the
swapchain depth. `OBSCUR_LYON_SAMPLES=1` uses single-sample rasterization for
frames without compositing groups or backdrop filters. Those effects still use
4× MSAA. The diagnostic setting may make edges visibly jagged. The default is
4× MSAA.

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

A painter is registered under a **symbol** the node names itself
(`UiRuntime::register_canvas`, or `App::painter_id`) or under a **name** a
stylesheet's `-obscur-painter` reaches (`register_named_canvas`, or
`App::painter`). The example below takes the symbol route because a `<canvas>`
built in Rust names its own painter; either is reachable from `App` without
touching the runtime by hand.

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
| display list, snapshot, scene clones | **0** — a `SceneImage` holds a refcounted `SharedPixels` |
| Vello | **0** — the `SharedPixels` is wrapped in a `Blob`, a refcount bump; then the GPU upload |
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

The three Vello caches this section used to describe as unwired scaffolding —
`GlyphCache`, `PathCache`, `TextureCache` — **have been deleted**, all 40 files
of them. They were unfillable by construction: `vello_encoding` keeps and prunes
its own glyph cache, `vello` holds the image atlas, and `PathCache` had no key at
all, because `obscur_scene::Path` holds floats and cannot derive `Hash`. There is
no `cache/` module in `obscur-render-vello` to look for.

The decoded-image cache on the runtime *is* live and bounded —
`set_image_cache_budget`, `set_image_eviction_policy` (default
`VisibilityAware`) and `image_cache_statistics` on
[03 — The runtime](03-runtime.md#resources-and-caches).
