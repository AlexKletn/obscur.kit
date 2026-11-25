Renders `segments` in order onto a fresh `width` × `height` frame and
returns it.

Each [`Content`](SceneSegment::Content) run draws over the accumulated
premultiplied frame. Vello samples that frame while writing to a separate
texture, then the renderer replaces the accumulator with the premultiplied
result. Ordinary filter layers composite
at the end of their segment, and each [`Backdrop`](SceneSegment::Backdrop)
boundary frosts the frame in place via [`GaussianBlur::render_backdrop`].
The renderer reuses the Gaussian pipelines across frames.

# Errors

[`RendererError::Backend`] on a Vello rasterization failure.
