Take the surface onto an acquire thread and start serving requests.

The surface moves: after this, resizing and reconfiguring go through
[`resize`](Self::resize), because the thread owns it.
