Hand-written because [`tiny_skia::Pixmap`] is not [`Debug`]; the pixel
buffer is summarized by its presence rather than dumped, and the host's
diagnostic sink is elided (it is a trait object).
