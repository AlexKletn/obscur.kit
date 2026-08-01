The root of a component's desired declarative output for one render.

A component may return zero, one, or many top-level nodes, so `View` wraps a
root [`ViewFragment`] rather than a single node. The whole structure is
plain data (spec §2.2): cloneable, comparable, and free of retained-tree
references, runtime handles, and closures, which keeps it suitable for
reconciliation and later lowering to UI IR (spec §2.15).

Because nodes carry [`super::ViewValue`] (which contains an `f64`), this
type derives only `Debug`, `Clone`, and `PartialEq`.
