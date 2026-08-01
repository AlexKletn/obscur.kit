Writes `body` as a framed LSP message: a `Content-Length` header (the body's
**byte** length), the blank-line separator, then the body — and flushes.

# Errors

Propagates any write or flush error from the underlying writer.
