A wheel notch's payload: a delta and a position.

The third kind, and it earns its own variant for the reason the variants exist
at all — a wheel carries none of §2.4's nine pointer facts and none of a key's
physical/logical pair. Carrying it as a `Pointer` would have meant synthesising a
`Move` that never happened; see [`WheelEvent`](crate::pointer::WheelEvent).
