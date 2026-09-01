Rasterise and blit into an image somebody else acquired.

The half of presenting that does not block. A host that reads input on its loop
thread fetches the image through a
[`SurfaceAcquirer`](crate::renderer::SurfaceAcquirer) and calls this, so the
wait for the compositor happens somewhere the input path cannot feel it.

[`present_to_surface`](Self::present_to_surface) is this plus the acquire, and
is only correct where blocking the caller is.

# Errors

`RenderFailed` where the image has no extent, `DeviceCreation` where no GPU
device is attached, and whatever rasterising the display list reports.
