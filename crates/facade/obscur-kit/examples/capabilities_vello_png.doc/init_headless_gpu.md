Acquires a **headless** wgpu device + queue (no surface). Returns [`None`] when
no compatible adapter/device is available, so the caller exits cleanly.
