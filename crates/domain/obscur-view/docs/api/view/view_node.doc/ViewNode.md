A single node in a component's desired declarative view tree.

The desired tree is cloneable, comparable, and free of retained-tree
references (`NodeId`) and runtime handles. Rust virtual lists may carry a
shared row builder that is consumed before reconciliation. Updates
are expressed by producing a new desired tree rather than mutating a
retained one (spec §2.2). Keyed identity used for reconciliation lives here
as an optional [`ViewKey`]; meaningful structure and text are both real
nodes (spec §4.16).

Because element props carry [`super::ViewValue`] (which contains an `f64`),
this enum derives only `Debug`, `Clone`, and `PartialEq`.
