A registered drop zone that a drag may be released onto (spec §6.21 typed
target).

A drop target names the retained [`node`](DropTarget::node) that accepts
drops, the MIME formats it [`accepts`](DropTarget::accepts) (an empty list
meaning "accept any"), and the [effects](DragEffect) it permits. During a
drag the session consults the target under the pointer to decide whether the
current payload is acceptable and which effect the drop would carry.
