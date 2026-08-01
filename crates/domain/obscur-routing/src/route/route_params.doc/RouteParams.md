The named parameters captured when a [`RoutePattern`] matches a path
(spec §6.13).

Parameters are stored as string key/value pairs in a [`BTreeMap`], which
keeps iteration in deterministic (sorted-by-key) order regardless of capture
order. The strings are the raw path segments captured by `:name` and
`*name` pattern segments.

The "typed params" feature of §6.13 is provided by
[`get_parsed`](RouteParams::get_parsed), which parses a captured string into
any [`FromStr`] target on demand.

[`RoutePattern`]: crate::route::RoutePattern
