End-to-end: the `Table` widget's own nodes are **visible to CSS**, and the UA
stylesheet's `table`, `th`, `td` and `tr:hover` rules reach them (spec §4.13,
§4.14).

# What this closes

A third of the UA sheet matched nothing. `Table::view` builds real element
nodes for its rows and cells, but they carried a **private** interned property
— `Table::PROP_ROLE`, whose value was the text `table`, `row` or `cell`. That
is not a class and not a local name, so no part of selector matching could see
it: the sheet's four table rules were dead against every table a program built,
while matching a hand-written `<table>` perfectly. One vocabulary, styled in
one place, meaning two different things depending on which route produced the
tree.

The parts name themselves now, in HTML's own vocabulary and under the reserved
`LOCAL_NAME` property the matcher reads as an element's *type* — `table`, `tr`,
`th`, `td`. `PROP_ROLE` is gone rather than kept beside them: nothing read it,
and everything it distinguished the local name distinguishes better. A header
cell is a `th` and a data cell a `td`, which the shared `TABLE_CELL_ID` cannot
express and which the sheet styles differently.

The accessibility tree gets the same fix for free and it is not a coincidence:
`SemanticBuilder::role_for_local_name` derives a §7.1 role from an element's
local name, and already mapped `table`/`tr`/`td`. A table's rows and cells used
to be announced as unnamed groups; `semantic_builder.rs`'s own unit tests pin
the roles, including the `th` entry this needed.

# What is asserted here

Computed values off a real frame, never rendered text. Text would prove the
widget built a tree and say nothing about whether the cascade can see it —
which is exactly how this survived.
