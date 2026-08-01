Cached offscreen render target, reused while the extent holds.

Vello renders into an offscreen and the result is blitted to the surface
(the surface cannot carry the STORAGE_BINDING usage Vello needs). That
target used to be allocated **per frame** — a full-window RGBA texture
created and dropped 60-120 times a second, which dominated present. Vello
clears it to `base_color` on every render, so reuse shows nothing stale.
