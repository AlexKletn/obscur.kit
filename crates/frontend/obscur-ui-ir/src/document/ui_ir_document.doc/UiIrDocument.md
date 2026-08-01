A complete compiled UI document: a versioned IR tree with its source map and
declared capability requirements (spec §2.5).

A document bundles everything needed to interpret a compiled UI: the
[`version`](UiIrDocument::version) it was written against, the
[`root`](UiIrDocument::root) IR node, a [`source_map`](UiIrDocument::source_map)
resolving node ids back to source spans for diagnostics, and the set of
[`required_capabilities`](UiIrDocument::required_capabilities) the host must
grant for the document to run.
