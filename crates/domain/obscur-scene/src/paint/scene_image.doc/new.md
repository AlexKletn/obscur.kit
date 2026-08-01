Creates an image from `width × height` RGBA8 `pixels` with the given
[`AlphaMode`], or `None` when the buffer length does not equal
`width × height × 4` (the byte-length guard: four bytes per pixel).

The pixels are taken as an [`Arc`] so no copy is made; callers with an
owned `Vec<u8>` can pass `Arc::from(bytes)`.
