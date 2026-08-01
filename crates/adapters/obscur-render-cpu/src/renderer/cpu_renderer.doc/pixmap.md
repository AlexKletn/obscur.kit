The pixels of the most recently rendered frame, or `None` before the
first successful [`Renderer::render`].

This is the surface a golden-image test inspects: its dimensions match the
last render target, and its contents are the deterministic rasterization of
that frame's display list (spec §5.2).
