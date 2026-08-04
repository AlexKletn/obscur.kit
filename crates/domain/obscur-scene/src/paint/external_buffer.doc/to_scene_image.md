Repacks the buffer into the RGBA8 the scene speaks.

This is where the copy happens: padding is dropped, channels are put in order,
and an opaque layout gets an alpha of 255. Called once per frame for a live
source, which for a 1080p frame is a few milliseconds of memory traffic — the
price of a CPU pixel pipeline, and the reason a producer that can already speak
tightly-packed RGBA8 should.

# Panics

Never in practice: the repacked buffer is exactly `width × height × 4` bytes by
construction, which is the only thing [`SceneImage::new`](super::SceneImage::new)
can refuse it for.
