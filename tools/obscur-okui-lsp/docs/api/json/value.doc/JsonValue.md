A JSON value: the seven shapes the LSP wire protocol needs.

Objects preserve insertion order (they are backed by a `Vec` of pairs), so
serialization is deterministic — which keeps round-trip tests and on-the-wire
output stable.
