Parses `source` as a selector list.

# Errors

Returns a [`CssParseError`] with kind
[`InvalidSelector`](CssParseErrorKind::InvalidSelector) on malformed
input (an empty compound, an unterminated `[ … ]`, and so on).
