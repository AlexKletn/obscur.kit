Attaches the experimental Sparse Strips renderer to a wgpu device and queue.
The returned renderer uses the same surface and readback entry points as classic Vello.

Call this when constructing the renderer. Images, backdrop blur, masks and layer filters
are currently omitted. Outset shadows are supported.
