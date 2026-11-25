Requests that the runtime re-render this component.

This only raises a flag; it performs no rendering and schedules no work.
The runtime observes the flag via [`needs_update`](Self::needs_update)
after the current call returns and decides what to do.
