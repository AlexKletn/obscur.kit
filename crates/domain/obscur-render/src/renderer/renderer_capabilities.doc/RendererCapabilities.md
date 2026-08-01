The capability profile a [`Renderer`](crate::renderer::Renderer) backend
advertises (spec §5.2).

Because the renderer abstraction hides backends of very different power
(spec §5.2) — from a software/headless rasterizer to a GPU platform backend —
callers introspect this profile to pick effects and fallbacks before
submitting work. [`Self::custom_shaders`] additionally gates the trusted
user-shader extensions of spec §5.3.
