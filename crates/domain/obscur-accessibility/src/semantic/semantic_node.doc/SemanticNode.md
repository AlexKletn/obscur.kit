One node of the semantic accessibility tree (spec §7.1).

A semantic node describes a single retained-tree node — identified by its
[`node`](Self::node) [`NodeId`] — for assistive technology. It bundles the
node's [`role`](Self::role), accessible [`name`](Self::name) and
[`description`](Self::description), [`state`](Self::state),
[`value`](Self::value), supported [`actions`](Self::actions), typed
[`relations`](Self::relations) to other nodes, on-screen
[`bounds`](Self::bounds) for hit-testing and highlighting, and a
[`decorative`](Self::decorative) flag.

Platform adapters project this data onto UI Automation, AX, AT-SPI, or web
accessibility APIs. A node that [`is_ignored`](Self::is_ignored) is not
exposed to assistive technology at all.
