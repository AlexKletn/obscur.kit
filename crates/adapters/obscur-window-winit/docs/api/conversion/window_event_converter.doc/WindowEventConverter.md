A stateless converter from winit's native window events to the domain's
[`WindowEvent`] (spec §6.1, §2.7).

This is a **pure** converter: it holds no event-loop state, so its output
depends only on its input. That determinism is the crux of the adapter's
correctness — the §5.2/§6.1 determinism boundary requires that identical
native events always yield identical domain events.

It handles only lifecycle/geometry/focus/scale/occlusion events. Input events
(keyboard and pointer) are routed by the dedicated
[`KeyboardConverter`](super::KeyboardConverter) and
[`PointerConverter`](super::PointerConverter); this converter maps them — and
any other unhandled winit event such as redraw requests — to `None`.
