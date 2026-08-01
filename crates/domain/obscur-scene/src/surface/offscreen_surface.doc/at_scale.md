Creates an offscreen render target of `size` logical pixels at `scale`.

A readability alias for [`Self::new`] that reads naturally at call sites,
e.g. `OffscreenSurface::at_scale(size, ScaleFactor::new(2.0))`.
