A request describing what to capture and which regions to protect
(spec §6.17).

A request is a pure value: a [`source`](Self::source) naming the target and a
set of [`protected_regions`](Self::protected_regions) the host must
exclude/redact because they are privacy-sensitive or capture-protected
(spec §6.17). Whether the capture succeeds is the host's decision, reported via
[`CaptureResult`](crate::capture::CaptureResult).
