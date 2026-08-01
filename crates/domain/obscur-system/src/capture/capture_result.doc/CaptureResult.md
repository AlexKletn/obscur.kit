The outcome a [`CaptureProvider`](crate::capture::CaptureProvider) returns for a
[`CaptureRequest`](crate::capture::CaptureRequest) (spec §6.17).

The four outcomes mirror the spec's vocabulary: the target was captured, was
denied by permission policy, is unsupported on this host, or failed. Only
[`Captured`](Self::Captured) carries a [`CapturedImage`]; [`Failed`](Self::Failed)
carries a human-readable reason.
