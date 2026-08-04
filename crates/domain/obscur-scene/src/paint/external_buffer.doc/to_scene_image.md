Repacks the buffer into the RGBA8 the scene speaks.

This is where the copy happens: padding is dropped, channels are put in order,
and an opaque layout gets an alpha of 255. Measured on a 1080p frame, 2.4 ms for
BGRA with padded rows and 0.9 ms for a buffer that is already tightly-packed
RGBA8 — the latter being one `memcpy`, which is the floor and the reason a
producer that can already speak that layout should.

The repacked buffer is MOVED into the image rather than converted to an
`Arc<[u8]>`; see [`SharedPixels`](super::SharedPixels) for why that distinction
was worth six milliseconds a frame.

# Panics

Never in practice: the repacked buffer is exactly `width × height × 4` bytes by
construction, which is the only thing [`SceneImage::new`](super::SceneImage::new)
can refuse it for.
