The result of running the measure and arrange phases for a node/subtree
(spec §4.3).

Besides the produced [`LayoutBox`], it carries whether the produced geometry
differs from the previous pass. That flag is the signal for **upward**
invalidation propagation: §4.3 propagates up only when output geometry
changes, so an unchanged result can stop propagation at its context
boundary.
