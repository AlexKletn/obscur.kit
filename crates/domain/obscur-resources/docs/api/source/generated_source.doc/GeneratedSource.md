A resource produced by a generator rather than loaded from stored bytes (spec
§5.16 generated source).

A generated source names the `generator` (its stable kind, for example
`"solid-color"` or `"noise"`) and carries an opaque `descriptor` — a spec
string the generator interprets as its parameters. It is a pure description:
the actual bytes are produced later when a loader runs the named generator.

Because equal `(generator, descriptor)` pairs compare and hash equal, identical
generated requests deduplicate in the hub (spec §5.15); [`Self::dedup_key`]
exposes that identity as a single string for keying.
