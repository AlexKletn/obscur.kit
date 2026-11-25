Rasterizes `submission`'s display list into a CPU bitmap sized to `target`.

Walks the commands in paint order, filling or stroking each path with the
solid color resolved from its paint and honoring the clip stack. On success
the rasterized pixels are retained (see [`CpuRenderer::pixmap`]) and the
scene generation that "reached the screen" is echoed.

# Errors

Returns [`RendererError::InvalidTarget`] when the target has zero area or
its pixel buffer cannot be allocated (a zero or too-large dimension). A
malformed or unbalanced display list never errors and never panics: stray
pops are ignored and unrepresentable commands degrade to a documented
best-effort (see the module docs).
