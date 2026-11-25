Every [`FrameReason`] in a fixed, deterministic order.

The index of a reason in this array is its tie-break ordinal (see
[`reason_ord`]); the array also serves as the reverse lookup in
[`reason_from_ord`]. Keeping it in one place ties the two directions
together.
