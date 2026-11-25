Resolves the [`ComputedStyle`] of every node reachable from the tree root,
inheriting down from each node's parent (spec §4.14).

Returns an empty map when the tree has no root. Otherwise the tree is
walked parent-first and each node is resolved against its cascaded
[`StyleInputs`] and its parent's already-computed style.

`viewport` supplies the `vw` / `vh` bases every computed `<length>` is
absolutized against (spec §4.5).

`metrics` supplies the `ch` base by measuring the `0` glyph. It is optional
only because a caller may not own a shaper; one that does should pass it, or
every `ch` on the tree stays symbolic and reaches paint as `0` — see
[`ResolutionContext::with_metrics`].
