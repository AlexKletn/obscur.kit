A handle for inserting further sources — timers, other file descriptors —
into the same loop the surface runs on.

This is what keeps a host from needing a second thread to own a timer: one
loop, one place events come from.
