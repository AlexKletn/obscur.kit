Validates every element in `document`, recursively, reporting each handler
reference no declared command claims (spec §2.13).

Returns nothing when the surface is
[`Unchecked`](CommandSurface::Unchecked) — including the empty-handler
diagnostic, which is a statement about the declared surface too: with no
declaration, a document's bindings are simply not this pass's business.
