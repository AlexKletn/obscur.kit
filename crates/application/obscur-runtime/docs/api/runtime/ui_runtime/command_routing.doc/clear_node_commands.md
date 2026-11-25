Removes every per-node handler **and** every §2.3 listener registered at
`node`, returning how many registrations were removed in total.

A node's registrations are not reaped automatically when it leaves the
tree — the runtime does not know whether the app means to re-register on
the node's replacement — so an app that registers per-node handlers on a
churning subtree calls this when it tears the subtree down. Leaving them
is safe but unbounded: an unreachable entry can never fire, because the
route can only name nodes reachable from the origin.

Listeners are counted per listener, not per `(node, command)` key: a node
carrying three listeners for one command reports three, because three
closures were dropped.
