The outcome of compiling a `.okui` document (spec §2.15).

# Policy

Every pass's diagnostics are accumulated (the pipeline never bails on the
first error), so `diagnostics` may hold several independent failures. The
`document` is `Some` when lowering could produce UI IR and `None` when a hard
lowering error (an empty document or a tagless element) prevented it — even
then, all earlier diagnostics are still reported. `source_map` is the lowered
document's map, or empty when no document was produced.
