A single node in a component's desired declarative view tree.

The desired tree is *plain data*: cloneable, comparable, and free of any
retained-tree references (`NodeId`), runtime handles, or closures. Updates
are expressed by producing a new desired tree rather than mutating a
retained one (spec §2.2). Keyed identity used for reconciliation lives here
as an optional [`ViewKey`]; meaningful structure and text are both real
nodes (spec §4.16).

Because element props carry [`super::ViewValue`] (which contains an `f64`),
this enum derives only `Debug`, `Clone`, and `PartialEq`.
