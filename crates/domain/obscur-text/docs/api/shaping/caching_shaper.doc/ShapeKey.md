The full cache key: everything a [`ShapingRequest`] carries that can change
the shaped output.

The `f32` font size is stored via [`f32::to_bits`] so the key can derive
`Eq` and `Hash` (a raw `f32` cannot). Face identity is captured by family,
attributes, and the cheap [`FaceSourceKey`]; text, direction, and the
optional language hint round it out. Two requests with the same key produce
byte-identical shaped runs, and any observable difference lands in a
different bucket.
