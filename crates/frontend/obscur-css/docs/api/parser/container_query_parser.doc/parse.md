Parses `source` as a container-query prelude (without the `@container`
keyword).

# Errors

Returns a [`CssParseError`] with kind
[`InvalidAtRule`](crate::error::CssParseErrorKind::InvalidAtRule) on
malformed syntax.
