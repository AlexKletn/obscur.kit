A single recoverable diagnostic produced while parsing `.okui` source
(spec §2.9).

A `ParseError` records *what* went wrong ([`ParseErrorKind`]), a
human-readable `message`, and the byte `offset` in the source where the
problem was detected. Errors never abort the parse: they are collected
alongside a lossless tree so tooling can report every problem at once.
