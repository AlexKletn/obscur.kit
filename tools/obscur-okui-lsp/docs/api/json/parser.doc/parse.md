Parses `input` into a [`JsonValue`], or returns a [`JsonError`] locating the
first problem.

Accepts the full JSON grammar the LSP needs: objects, arrays, strings (with
`\"`, `\\`, `\/`, `\b`, `\f`, `\n`, `\r`, `\t` and `\uXXXX` escapes, including
UTF-16 surrogate pairs), numbers, `true`/`false`/`null`. Trailing content
after a single top-level value is rejected.

# Errors

Returns a [`JsonError`] when `input` is not a single well-formed JSON value.
