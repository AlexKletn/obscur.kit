Renders `segments` in order onto a fresh `width` × `height` frame and
returns it.

Each [`Content`](SceneSegment::Content) run is rendered to a transparent
offscreen and composited onto the accumulating frame `T` — its own
`filter: blur` layers composited on top just as the non-segmented path does
— and each [`Backdrop`](SceneSegment::Backdrop) boundary frosts `T` in place
under the panel's rounded border box via the owned
[`GaussianBlur::render_backdrop`]. The same owned [`GaussianBlur`] pipelines
are reused across frames, so only the first segmented frame builds them.

# Errors

[`RendererError::Backend`] on a Vello rasterization failure.
