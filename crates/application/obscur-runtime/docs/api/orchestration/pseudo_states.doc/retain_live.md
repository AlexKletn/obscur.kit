Drops the state of every node the tree no longer holds.

A `NodeId` is generational, so a stranded entry can never be mis-read by the node that takes its slot — the cost is memory, plus the two walks `RestyleScope::changed_states` makes over this map and the clone `StyleKey` takes of it every frame. A pointer event replaces the whole map from the live hover chain and hides the growth; a runtime driven only through `set_pseudo_state` has nothing that ever removes an entry.
