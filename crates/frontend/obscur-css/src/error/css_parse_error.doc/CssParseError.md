A syntactic error produced while parsing CSS (spec §4.13).

Pairs a [`CssParseErrorKind`] with a human-readable `message` and the
[`SourceSpan`] the error covers. It implements [`std::error::Error`] so it
composes with the standard error ecosystem while remaining I/O-free.
