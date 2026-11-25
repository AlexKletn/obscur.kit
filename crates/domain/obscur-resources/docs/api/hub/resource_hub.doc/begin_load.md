Begins loading `handle`, returning the version a job should be stamped
with (spec §5.15).

Sets the state to [`ResourceState::Loading`] with indeterminate progress
and returns the resource's current authoritative [`ResourceVersion`]; the
runtime dispatches a load job carrying this version, which
[`ResourceHub::complete`] later checks for staleness. Returns `None` if the
handle is unknown.
