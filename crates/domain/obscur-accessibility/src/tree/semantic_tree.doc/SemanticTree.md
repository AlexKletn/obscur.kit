The semantic accessibility tree linked to the retained UI tree (spec §7.1).

Accessibility is projected through a **semantic tree** whose nodes are keyed
by the retained-tree [`NodeId`]s they describe. Platform adapters target UI
Automation, AX, AT-SPI, or web accessibility APIs from this shared structure;
they never touch the retained tree directly.

The tree owns a [`SemanticNode`] per id, an ordered child list per parent,
an optional [`root`](Self::root), and a monotonic [`version`](Self::version)
counter that is bumped on every structural or content mutation. The version
is a plain counter — never a wall-clock value — so a [`SemanticSnapshot`]
taken from this tree can be compared against the live version to detect
staleness (spec §7.1).

[`SemanticSnapshot`]: crate::tree::SemanticSnapshot
