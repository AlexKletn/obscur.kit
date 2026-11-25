A raster-image [`ResourceDecoder`] backed by the `image` crate (spec §5.16).

[`ImageDecoder`] decodes encoded raster bytes (PNG, JPEG) into a CPU
representation of tightly-packed, non-premultiplied RGBA8 pixels in row-major
order, and records the intrinsic pixel size and aspect ratio the runtime needs
for object-fit layout (spec §5.16). It enforces a maximum per-side dimension as
the spec §5.16 safety limit, rejecting oversized images with
[`ResourceError::LimitExceeded`] before allocating their pixel buffer.
