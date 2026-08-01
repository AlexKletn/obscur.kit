A monotonic frame counter that uniquely labels each produced frame.

`FrameId` is a lightweight newtype over `u64`. The runtime starts at
[`FrameId::FIRST`] and advances by [`FrameId::next`] once per produced frame,
so a larger id always denotes a later frame (until the counter wraps, which
is astronomically distant for a `u64` frame clock).

The type is a plain value: `Copy`, totally ordered, hashable, and `Default`
(equal to [`FrameId::FIRST`]), which makes it usable as a map key or in an
ordered log without any allocation.
