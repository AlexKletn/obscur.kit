Returns whether the instance is currently participating in the tree.

Active means either settled ([`LifecycleState::Mounted`]) or mid-update
([`LifecycleState::Updating`]); a `Created` instance has not entered the
tree and an `Unmounted` one has left it.
