The host-driven frame entry: sets the host clock to `host_clock_ms` and
asks the kit for one frame's worth of render commands. It returns rather
than blocking, and never enters a loop.
