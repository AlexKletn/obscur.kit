The root of a component's desired declarative output for one render.

A component may return zero, one, or many top-level nodes, so `View` wraps a
root [`ViewFragment`] rather than a single node. It is cloneable, comparable,
and free of retained-tree references and runtime handles (spec §2.2). A
Rust virtual-list marker may own a row builder used before reconciliation.
Those builders are specific to the Rust view and cannot be encoded as UI IR.

Because nodes carry [`super::ViewValue`] (which contains an `f64`), this
type derives only `Debug`, `Clone`, and `PartialEq`.
