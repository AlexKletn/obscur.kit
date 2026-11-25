Reads a single framed message body from `reader`.

Consumes the header block (parsing `Content-Length`, ignoring other headers
such as `Content-Type` case-insensitively), then reads exactly that many
bytes and decodes them as UTF-8. Returns `Ok(None)` at a clean end of stream
(no bytes before the headers).

# Errors

Returns an error if the stream ends mid-message, the `Content-Length` header
is missing or unparsable, or the body is not valid UTF-8.
