The replaceable port for screen / surface / node capture (spec §6.17).

[`CaptureProvider`](super::CaptureProvider) is the hexagonal port through which the domain asks the
host to satisfy a [`CaptureRequest`] and receives a
[`CaptureResult`]. Permission and capability are honest: a provider that cannot
capture a source answers [`CaptureResult::Unsupported`] (or
[`CaptureResult::Denied`]) rather than panicking, and can be pre-checked with
[`can_capture`](CaptureProvider::can_capture). The colocated
[`NullCaptureProvider`](super::NullCaptureProvider) and [`StubCaptureProvider`](super::StubCaptureProvider) are reference
implementations of that contract.
