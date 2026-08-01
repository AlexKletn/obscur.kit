An ordered sequence of [`ViewNode`]s.

A fragment is the output shape of a component render and of collection or
conditional constructs (`For`/`If`, spec §2.11), which produce nodes only
when their rendered output requires them. Recursion in the desired tree
arises because an element node's `children` is itself a `ViewFragment`.

Because nodes carry [`super::ViewValue`] (which contains an `f64`), this
type derives only `Debug`, `Clone`, and `PartialEq`.
