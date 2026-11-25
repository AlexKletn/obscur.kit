A stateless converter from winit's native mouse input to the domain's
[`PointerEvent`] model (spec §4.7, §2.7).

These are **pure**, per-event conversions with no event-loop state: each maps
one winit sub-value (a button, an element state, a cursor position) to the
matching domain value. The converters here intentionally do *not* track the
held-button set across events — full button-state tracking is the backend's
job, which threads a running [`PointerButtons`](obscur_interaction::pointer::PointerButtons)
through these per-event conversions. Every event produced here uses
[`PointerDevice::Mouse`] and the conventional [`PointerId::MOUSE`].
