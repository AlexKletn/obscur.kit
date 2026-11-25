The classification of a [`ParseError`](super::ParseError) (spec §2.9).

The parser is *error-tolerant*: it never hard-fails, so each recoverable
problem is recorded as one of these kinds and parsing continues over a
lossless tree.
