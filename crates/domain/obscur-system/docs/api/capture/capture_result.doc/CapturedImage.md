A descriptor of successfully captured pixels (spec §6.17).

`CapturedImage` names the [`size`](Self::size) and pixel [`format`](Self::format)
of a capture; the actual bytes live in an interface adapter, never in the pure
domain. It is the payload of [`CaptureResult::Captured`](super::CaptureResult::Captured).
