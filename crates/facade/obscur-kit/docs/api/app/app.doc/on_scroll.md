Registers a handler for wheel scrolling the kit did not consume.

A `overflow: scroll` container inside the kit takes the wheel first; this is
what is left over, in logical pixels, positive when the content should move
down. An app that windows its own list owns its scroll position and needs
this to move it.

Not a §2.3 event either: a wheel notch no container consumed is offered to this
closure by the window runner, not dispatched as a command, so there is nothing
to observe or prevent. It stays an `FnMut(f32)` because that is the whole
payload it has — leftover pixels along the block axis.
