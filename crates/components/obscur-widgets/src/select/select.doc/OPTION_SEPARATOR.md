The character [`PROP_OPTIONS`] joins option labels with: U+001F INFORMATION
SEPARATOR ONE, the ASCII unit separator.

Chosen because it is the character whose *only* meaning is this one, and
because no label a caller can type contains it — the round trip through
[`PROP_OPTIONS`] is therefore total for every list a [`Select`](Self) can
hold. It is deliberately not a comma: the authored `options` attribute is
comma-separated because a human writes that one, and a human cannot write
this one.

[`PROP_OPTIONS`]: Self::PROP_OPTIONS
