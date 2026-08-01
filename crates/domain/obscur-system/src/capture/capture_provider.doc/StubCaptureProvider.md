A deterministic, recording [`CaptureProvider`] for tests (spec §6.17).

`StubCaptureProvider` accepts every source: [`can_capture`](Self::can_capture)
is always `true`, and [`capture`](Self::capture) records the request in
[`last`](Self::last) and returns a fixed 1x1 `rgba8`
[`CaptureResult::Captured`]. Tests inspect the recorded request to assert what
the domain attempted, without touching a real capture pipeline.
