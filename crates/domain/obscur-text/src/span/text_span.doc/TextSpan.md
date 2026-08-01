A styled run of text (spec §5.10): a [`TextRange`] paired with the
[`TextStyle`] applied to those bytes.

Rich text is described by a sequence of non-overlapping spans; managing that
sequence (ordering, coalescing, splitting) is the job of the buffer and
layout modules built later. A single `TextSpan` models just one contiguous
styled range.

Because [`TextStyle`] holds `f32` fields, `TextSpan` derives only `Debug`,
`Clone`, and `PartialEq` (floats preclude `Eq`).
