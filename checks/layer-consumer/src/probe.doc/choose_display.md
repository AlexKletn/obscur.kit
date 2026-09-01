Resolve a connector name to a `DisplayId`, before anything is opened.

The ordering problem `App::displays` exists to solve, written out: settings must
be complete before `run`, and an id only means something once a backend has
enumerated its outputs.

# Errors

Any layer failure other than "unsupported", which is folded into
[`NoDesktop`](DisplayChoice::NoDesktop) instead.
