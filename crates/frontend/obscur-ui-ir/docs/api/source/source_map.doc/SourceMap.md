A map from IR node ids to the [`SourceReference`]s they originated from
(spec §2.5).

Diagnostics raised against the IR carry a node id; the source map resolves
that id back to a span in the original source (for example the `.okui` file)
so errors can point at the code the author wrote.

Node ids are stored as raw `u64` keys. The document module is what assigns
and interprets these ids; keeping them as plain integers here lets the source
map stay independent of the document types (no dependency cycle) while still
keying on the same value. A [`BTreeMap`] gives deterministic iteration order,
which keeps serialized source maps and diagnostics stable across runs.
