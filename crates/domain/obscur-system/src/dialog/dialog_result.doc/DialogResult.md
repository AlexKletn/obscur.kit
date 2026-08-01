The outcome a [`DialogProvider`](crate::dialog::DialogProvider) returns for a
[`DialogRequest`](crate::dialog::DialogRequest) (spec §6.9).

The result is a pure value: each variant reports what the user chose, or that
the interaction ended without a choice. Successful file dialogs yield opaque
[`DocumentHandle`]s / [`DirectoryHandle`]s rather than paths (spec §6.10), so
the outcome carries no filesystem access of its own. [`Denied`](Self::Denied)
and [`Unsupported`](Self::Unsupported) let a caller fall back to another
provider in a chain.
