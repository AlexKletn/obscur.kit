Takes what the last frame's dispatch phase produced, in the order the
events were queued.

Empty when the last frame dispatched nothing. Taken rather than borrowed
because each frame's dispatches replace the previous frame's: a host that
does not read them is not accumulating a leak.
