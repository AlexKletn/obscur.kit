Parses `source` as a single declaration.

# Errors

Returns a [`CssParseError`]: [`ExpectedColon`](CssParseErrorKind::ExpectedColon)
if the `:` is missing, [`UnexpectedToken`](CssParseErrorKind::UnexpectedToken)
if the property name is missing, or
[`InvalidValue`](CssParseErrorKind::InvalidValue) if the value is empty or
malformed.
