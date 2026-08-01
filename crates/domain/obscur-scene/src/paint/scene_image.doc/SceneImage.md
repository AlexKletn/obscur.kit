A decoded raster image: `width × height` RGBA8 pixels plus how their alpha is
stored (spec §5.16).

This is the backend-independent value a
[`PaintCommand::DrawImage`](crate::command::PaintCommand::DrawImage) carries.
It holds only **raw bytes** — the domain never depends on an image codec or a
GPU/CPU image type — so a renderer is free to wrap the pixels into its own
texture (`peniko::Image`, `tiny_skia::Pixmap`, …). Pixels are laid out row by
row, top-to-bottom, four bytes (`R`, `G`, `B`, `A`) per pixel; the
[`AlphaMode`] states whether those channels are premultiplied by alpha.

The pixel buffer is stored behind an [`Arc`] so cloning a `SceneImage` — which
happens every time a display list is cloned — only bumps a refcount rather than
copying the (potentially large) pixel data.

# Pixels versus logical size

[`width`](Self::width) / [`height`](Self::height) are **pixel** counts. An
image loaded from a DPI variant (spec §5.16) carries more pixels than the
logical box it fills, so it is tagged with a [`scale`](Self::scale) and its
intrinsic size for layout/`object-fit` purposes is
[`logical_size`](Self::logical_size) — the pixel count divided by that
density. Sizing code must read `logical_size`; a renderer blitting the buffer
wants the raw pixel dimensions.

# Not yet modeled

A `SceneImage` describes a whole decoded bitmap drawn at a destination rect
(see [`PaintCommand::DrawImage`](crate::command::PaintCommand::DrawImage)). A
source sub-rectangle, tiling/`repeat`, nine-slice, and CSS `background-size`
keywords are future additions; today an image is stretched to fill its
destination rect.
