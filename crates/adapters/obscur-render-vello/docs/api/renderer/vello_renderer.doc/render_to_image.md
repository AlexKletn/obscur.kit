Renders `submission` fully **headless** into an offscreen texture and reads
the pixels back to CPU memory as tightly-packed RGBA8 (spec §5.6 offscreen
output).

This is the read-back counterpart to the port [`Renderer::render`] (which
leaves the result on the GPU) and to [`Self::present_to_surface`] (which
blits it to a window): it rasterizes into the same
[`VelloFrame::new_offscreen`] intermediate with
[`vello::Renderer::render_to_texture`], then copies that texture into a
mappable [`wgpu::Buffer`] — honoring wgpu's
[`COPY_BYTES_PER_ROW_ALIGNMENT`](wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) 256-byte
row stride — waits for the copy with a blocking [`Device::poll`], un-pads the
rows, and returns `(width, height, rgba8)`. It **does not** disturb the
on-screen path or the trait `render`; a host uses it to produce PNGs from a
display list without a window.

The returned bytes are **premultiplied** RGBA8, with straight-alpha output converted when needed.
They can be passed directly to `tiny_skia::Pixmap::from_vec`.

# Errors

Returns [`RendererError::DeviceLost`] when no GPU device is attached, and
[`RendererError::Backend`] on a zero extent, a Vello rasterization failure,
or a buffer-map / device-poll failure.
