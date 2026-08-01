Hosts a mounted component: this node's children are produced by that
component's `render`, **not** by the desired tree (spec §2.1, §2.2).

Reconciliation therefore does not recurse into this node's children — the
desired node names the component and carries its props, but says nothing
about what it renders, so descending would delete the component's output
on every frame. The runtime sets this when it mounts an instance and
clears it when it unmounts one.
