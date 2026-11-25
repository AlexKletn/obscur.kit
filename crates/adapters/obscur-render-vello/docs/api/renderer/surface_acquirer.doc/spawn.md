Take the surface onto an acquire thread and start serving requests.

The surface moves: after this, resizing and reconfiguring go through
[`resize`](Self::resize), because the thread owns it.

If the worker fails to start, the surface is released and the acquisition
handle stays idle. Retrieve the cause with [`take_startup_error`](Self::take_startup_error).
