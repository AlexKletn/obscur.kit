The format to present through: the surface's own, with any sRGB suffix removed.

Vello rasterises into an `Rgba8Unorm` texture whose bytes are **already
sRGB-encoded**. Copying those bytes into a view whose format carries the `Srgb`
suffix makes the hardware encode them again, and the result is every colour on
screen lighter than the one authored — barely visible on a pale design, ruinous
on a dark one.

Blitting through a view in this format instead copies the bytes as they are.
[`new`](Self::new) is what makes such a view legal, by listing the format in the
surface's `view_formats`.

Equal to the surface's own format where that one was never sRGB, so this is
always the right thing to ask for.
