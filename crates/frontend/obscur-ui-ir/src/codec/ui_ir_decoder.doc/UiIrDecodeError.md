A fault encountered while decoding a UI IR byte stream (spec §2.5).

Each variant names one way the input failed to be a well-formed IR stream in
the format documented on [`UiIrEncoder`](crate::codec::UiIrEncoder): a bad
header, a truncated buffer, or an unknown discriminant tag.
