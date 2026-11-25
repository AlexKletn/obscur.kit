Whether this frame's semantic projection could have changed, and so
whether it is worth building one.

`tree_changed` is the frame's own reconcile result, so the walk that
builds the projection runs only when the retained tree it reads actually
changed or when focus moved. A hover, animation or idle frame
re-publishes nothing, which is why the push can sit on the frame path at
all: the semantic tree is a full walk of the retained tree, and a screen
reader has no use for the same tree sixty times a second.
