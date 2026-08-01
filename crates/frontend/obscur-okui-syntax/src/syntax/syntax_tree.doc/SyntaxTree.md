A whole parsed `.okui` document as a lossless syntax tree (spec §2.8).

The tree owns a single root [`SyntaxNode`]. Because every leaf keeps its
exact source slice, [`SyntaxTree::text`] reconstructs the original source
byte-for-byte — the losslessness guarantee that lets tooling round-trip and
re-print `.okui` sources without data loss.
