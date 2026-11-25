Takes what the last dispatch phase produced, in the order the events were
queued.

Taken rather than borrowed because each frame's dispatches replace the
previous frame's: a host that does not read them is not accumulating a
leak.
