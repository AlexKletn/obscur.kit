Resolves the winning **cascaded value** for each property.

For every [`PropertyId`] present among `declarations`, the returned
[`ResolvedDeclarations`] holds the value of the declaration with the
greatest [`CascadePriority`] (see the type-level docs for the exact
§4.14 ordering).

Resolution is deterministic. When two declarations for the same property
compare *exactly* equal in priority — which cannot happen when
`source_order` is unique within a band — the later one in `declarations`
is kept.

Only known [`PropertyId`] declarations participate; custom-property
(`--*`) resolution is out of scope for this sub-step. No inheritance and
no CSS-wide-keyword resolution is applied here — the returned values are
cascaded values, not computed values.
