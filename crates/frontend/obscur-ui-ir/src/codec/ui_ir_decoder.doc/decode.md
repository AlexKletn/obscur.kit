Decodes `bytes` into a [`UiIrDocument`], reversing
[`UiIrEncoder::encode`](crate::codec::UiIrEncoder::encode).

# Errors

Returns a [`UiIrDecodeError`] if the header is wrong, the buffer is
truncated, a discriminant tag is unknown, a string is not valid UTF-8, or
a node uses a not-yet-constructible kind (`Conditional`/`Loop`).
