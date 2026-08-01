What a press on the focused editable resolved to — the answer
[`TextPointerSelection::press`] returns and the runtime acts on.

The runtime applies the selection through its own edit choke point and owns
the drag anchor, because the armed drag lives with the pointer sessions and
rule 2 of this crate's decomposition forbids this entity from holding them.
