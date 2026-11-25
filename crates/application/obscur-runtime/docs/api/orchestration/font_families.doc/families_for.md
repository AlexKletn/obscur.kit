Builds the [`FontFallback`] chain for a text node from its computed
`font-family` (§4.14, §5.10).

Reads [`PropertyId::FontFamily`] — a `FontFamilies` string list in author
order — and maps each entry through [`FontFamily::from_name`] (generic
keywords such as `sans-serif` become a `GenericFamily`, everything else a
named family), then deduplicates them into a chain via
[`FontFallback::from_families`]. When the property is absent (no authored
`font-family`) or its list is empty, the runtime's
[`FontFallback::default_chain`] is returned — byte-for-byte the behaviour
before per-node `font-family` resolution.

This single helper feeds **both** the layout measure/arrange path and the
scene compose path so the chain the box is measured with is the chain it is
painted with.
