Tracks which node, if any, has captured each pointer (spec §4.7).

A captured pointer routes all its subsequent events to its capturing node
instead of the hit-tested target: capture *overrides* hit-testing. The
registry maps each captured [`PointerId`] to the [`NodeId`] holding it,
backed by a [`BTreeMap`] for deterministic ordering.
