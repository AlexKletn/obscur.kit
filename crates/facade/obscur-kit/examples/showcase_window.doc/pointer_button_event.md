Assembles a domain button [`PointerEvent`] at the tracked cursor, reusing
[`PointerConverter::kind`] to map the winit [`ElementState`] to the domain
event kind (the converter exposes no builder for a button transition *at a
position*, so the event is built directly — as in `windowed_counter.rs`).
