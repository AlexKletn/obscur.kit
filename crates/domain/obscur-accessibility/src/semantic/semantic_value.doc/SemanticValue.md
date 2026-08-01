The value carried by an accessibility node (spec §7.1 value).

Different roles expose different value shapes: a text field carries
[`Text`](Self::Text), a toggle carries a [`Bool`](Self::Bool), and a range
control such as a slider or progress bar carries a bounded
[`Number`](Self::Number). [`None`](Self::None) is the absence of any value.

[`Number`](Self::Number) keeps its `min` and `max` alongside the value so a
platform adapter can report the range; [`ratio`](Self::ratio) normalises the
value into `[0, 1]`.
