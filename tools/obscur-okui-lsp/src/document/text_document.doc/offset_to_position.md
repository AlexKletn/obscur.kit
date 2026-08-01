Maps an absolute byte `offset` to a zero-based [`Position`].

Offsets past the end of the text are clamped to the end. The returned
`character` is a UTF-8 byte offset within the line (see [`Position`]).
