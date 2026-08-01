Parses `source` as a media-query prelude (without the `@media` keyword).

# Errors

Returns a [`CssParseError`] with kind
[`InvalidAtRule`](CssParseErrorKind::InvalidAtRule) on malformed syntax.
