The outcome of parsing a `.okui` source (spec §2.9).

A parse *always* produces a lossless [`SyntaxTree`] whose
[`text`](SyntaxTree::text) reproduces the source byte-for-byte, even when the
input was malformed. Any recoverable problems are collected into `errors`;
an empty `errors` list means the source parsed cleanly ([`is_ok`](Self::is_ok)).
