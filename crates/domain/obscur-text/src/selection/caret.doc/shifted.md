Returns this caret moved by `delta` bytes, saturating at the bounds of
`usize` so the offset never overflows or underflows. The affinity is
preserved.
