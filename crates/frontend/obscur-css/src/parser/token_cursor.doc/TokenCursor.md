A forward cursor over a token slice, shared by every sub-parser.

The cursor exposes whitespace-aware navigation: whitespace tokens remain in
the stream (selectors need them as descendant combinators) but can be skipped
on demand. It records the end-of-input byte offset so parsers can raise
[`CssParseError::unexpected_eof`](crate::error::CssParseError::unexpected_eof)
with an accurate location.
