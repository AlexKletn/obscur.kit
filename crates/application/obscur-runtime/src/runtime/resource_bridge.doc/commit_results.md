Applies every finished resource-load result waiting on the channel and
returns how many were accepted, **without** asking for a frame.

This is the §1.5 synchronization point itself, kept separate from the frame
request because the frame path needs the commit but not the request: a
frame is already running, and requesting a `ResourceCompletion` from inside
one would leave a pending reason behind and cost a redundant frame after
every load.

A **stale** result is dropped here rather than applied — the hub rules on
the version, not arrival order — and so is not counted.
