Creates an image from `width × height` RGBA8 `pixels` with the given
[`AlphaMode`], or `None` when the buffer length does not equal
`width × height × 4` (the byte-length guard: four bytes per pixel).

`pixels` is `impl Into<SharedPixels>`, so **pass an owned `Vec<u8>` directly** —
it is moved into an `Arc` and nothing is copied. Do *not* reach for
`Arc::<[u8]>::from(bytes)` first: that conversion allocates a second buffer and
copies into it, measured at 1080p as six milliseconds a frame to publish bytes
that were already sitting there. An `Arc<[u8]>` you already hold (a decoded
image shared with a cache) converts for free and should be passed as-is.
