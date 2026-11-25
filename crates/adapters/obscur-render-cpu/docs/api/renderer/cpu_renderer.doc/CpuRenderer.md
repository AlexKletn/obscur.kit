A software [`Renderer`] backend that rasterizes scenes to a CPU bitmap
(spec §5.2, §2.7).

The renderer owns no GPU or OS handle: it advertises a conservative software
capability profile (offscreen output, no MSAA/HDR/custom shaders) and renders
entirely on the CPU with `tiny-skia`. Each [`Renderer::render`] call
rasterizes into a fresh [`tiny_skia::Pixmap`], which it retains as
[`CpuRenderer::pixmap`] for headless golden-image comparison.
