Acquires a **headless** wgpu device + queue (no surface), driving the async
`request_adapter` / `request_device` to completion with `pollster`. Returns
[`None`] when no compatible adapter or device is available (a headless box with
no GPU), so the caller can exit cleanly rather than panic.
