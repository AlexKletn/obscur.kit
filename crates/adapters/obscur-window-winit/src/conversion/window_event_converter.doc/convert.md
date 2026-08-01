Converts a winit window `event` to the matching domain [`WindowEvent`],
or `None` when the event is routed elsewhere or ignored (spec §6.1).

The numeric casts to `f32` are lossless enough at UI scale: window sizes
and positions never exceed the precise integer range of `f32`, and scale
factors carry only a few significant digits.

# Known defect: `Resized` and `WindowProperties::size` disagree on units

`Resized` and `Moved` pass winit's **physical** pixels straight through —
winit types them `Resized(PhysicalSize<u32>)` (`winit-0.30.13/src/event.rs:161`)
and `Moved(PhysicalPosition<i32>)`. But a window is *created* from
`WindowProperties::size` in **logical** pixels:
`WindowAttributes::with_inner_size(LogicalSize::new(...))`
(`backend/winit_event_loop.rs:473`). Both ends are typed
[`Size`](obscur_geometry::Size), so nothing in the type system flags the
mismatch: a host that asks for a 480×240 window and reads back
`Resized(960.0, 480.0)` on a 2× display is seeing units change under one
type, not a resize.

This is **not fixed here**, and no test pins it — a test asserting the
current behaviour would be asserting the bug. It is invisible at scale 1,
which is why it survived: the physical passthrough is what a wgpu
swapchain wants, so the reference host in `windowed_counter` is correct
today *by coincidence of scale*, not by contract. Deciding it properly
means choosing whether the domain `Size` is logical (and dividing here,
which needs the per-window scale factor this converter is not given) or
physical (and converting at creation instead) — a domain-semantics call
beyond this converter, and one that wants a HiDPI display to verify.
