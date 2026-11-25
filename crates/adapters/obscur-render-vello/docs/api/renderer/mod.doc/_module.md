The Vello/wgpu renderer backend (spec §5.2, §2.7).

Implements the `Renderer` port of spec §5.2 on top of Vello and wgpu. Because
wgpu device creation is asynchronous and needs a real GPU, the *host* owns the
wgpu context and passes the device in (spec §2.7); the adapter never calls
`request_adapter`. A [`VelloRenderer`] therefore holds an **optional**
[`VelloDevice`] and returns an honest diagnostic when asked to render without
one.

- [`VelloRenderer`] — the `Renderer` implementation and its optional device.
- [`VelloDevice`] — a host-provided wgpu device/queue plus the Vello renderer.
- [`VelloSurface`] — an on-screen swapchain surface for windowed present.
- [`VelloFrame`] — the per-frame offscreen texture Vello rasterizes into.
- `filter_layer_pass` — the `filter: blur()` / colour-op sub-scenes, rendered
  to their own offscreens and composited back through the owned wgpu passes.
- `segmented_compositor` — the ordered content/backdrop compositing a
  `backdrop-filter` forces, since it frosts the frame accumulated so far.
