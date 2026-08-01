Configures the surface in one call: sets the device `scale` and derives the
layout viewport from the surface's **physical** extent.

The viewport divides by the *clamped* scale, so the two cannot disagree —
which is the whole reason this is one operation rather than two setters a
host must remember to pair. Returns what
[`set_device_scale`](Self::set_device_scale) reports.
