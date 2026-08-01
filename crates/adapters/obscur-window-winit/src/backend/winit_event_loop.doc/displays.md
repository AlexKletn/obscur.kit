The display topology as of the last enumeration, which is what the
backend's `displays()` hands out. Monitors, like windows, are only
reachable through an [`ActiveEventLoop`], so the pull-based port reads
this snapshot rather than the platform (spec §6.6).
