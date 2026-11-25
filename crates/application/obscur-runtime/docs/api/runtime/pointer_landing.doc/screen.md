The event's position in **content space**: the host's physical position
mapped through the inverse of the root render transform, before any
container's own scroll offset is folded in.

This is the space the clip test, the pan recognizer and the virtual-list
row lookup work in — everything that asks where the finger is *on screen*
rather than where it is in the document.
