Draws an image at an exact rectangle in this region's own coordinates.

The escape hatch under [`draw_buffer`](CanvasRegion::draw_buffer), for a caller
that already holds a [`SceneImage`](crate::paint::SceneImage) — an image decoded
once and drawn every frame costs nothing here — or that wants a placement no
[`BufferFit`](crate::paint::BufferFit) describes.
