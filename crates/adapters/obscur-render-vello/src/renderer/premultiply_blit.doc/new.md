Build the pipeline for one surface format.

The format is the surface's own with any sRGB suffix stripped (see
[`VelloSurface::blit_format`](super::VelloSurface::blit_format)): the pass must
not re-encode what Vello already encoded. Rebuilding recompiles the shader, so
the caller caches one per format.
