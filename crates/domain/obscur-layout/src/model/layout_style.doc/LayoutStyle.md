The computed layout input for a single node (spec §2.3, §4.1).

This is layout's own vocabulary, mapped from the style crate's computed style
by the runtime; the layout crate does not depend on `obscur-style`. All
lengths are fractional logical pixels and are preserved without rounding
(spec §4.6).

Grid, positioning, docking, and other formatting contexts are out of scope
for Phase 1 and are not represented here.
