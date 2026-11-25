An in-memory map from document URI to its open [`TextDocument`].

The store mirrors the LSP document-lifecycle notifications: `didOpen` →
[`open`](Self::open), `didChange` → [`change`](Self::change) (full-text
synchronization) and `didClose` → [`close`](Self::close). It is the single
source of truth the [`Analyzer`](crate::Analyzer) reads from.
