Cached offscreen render target, reused while the extent holds.

Vello renders into an offscreen and the result is blitted to the surface
(the surface cannot carry the STORAGE_BINDING usage Vello needs). That
target used to be allocated **per frame** — a full-window RGBA texture
created and dropped 60-120 times a second, which dominated present. Vello
clears it to `base_color` on every render, so reuse shows nothing stale.

**Every device-bound cache in this renderer, and what makes it stale.** Four
fields here hold GPU resources across frames — `frame_cache`, `blitter`, `blur`
and `color_matrix` — and one lives inside the blitter (its bind group). The
complete list of events that can make any of them wrong:

* **Extent change.** Only `frame_cache` cares, and `cached_offscreen` is where
  it is answered: a differing width or height builds a new `VelloFrame`. The
  blitter's bind group follows automatically, because the new frame carries a
  new view and the view is that cache's key.
* **Surface format change.** Only `blitter` cares. `present_to_texture_with`
  drops it, and its bind group goes with it.
* **Device loss.** Nothing answers this, because today nothing can raise it:
  `VelloDevice` is constructed exactly twice in the toolkit — `layer_runner` and
  `runner`, both at start-up — and never reconstructed. `SurfaceLost` is handled
  by skipping the frame, not by rebuilding the device, so a lost device today
  means a dead process. **If that ever stops being true, all five caches must be
  dropped together with the device**, and this paragraph is the list.
* **Scale change.** Reaches here only as an extent change; the renderer is told
  physical pixels and knows nothing of scale.

Nothing else is a staleness event. In particular the CONTENTS of the offscreen
change every frame and matter to none of them: these cache handles, not pixels.
