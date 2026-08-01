A reference, no-op [`CaptureProvider`] that captures nothing (spec §6.17).

`NullCaptureProvider` honestly reports that it can capture no source:
[`capture`](Self::capture) returns [`CaptureResult::Unsupported`] and
[`can_capture`](Self::can_capture) returns `false`. Its answers are correct
behavior — a documented reference for the honest-capability contract — not a
stub waiting to be filled in.
