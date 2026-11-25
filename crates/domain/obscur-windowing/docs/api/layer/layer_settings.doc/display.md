Which display to open on, or [`None`] to let the platform choose.

A [`DisplayId`] from
[`WindowBackend::displays`](crate::port::WindowBackend::displays) rather than a
name, because a name that identifies a monitor on one platform identifies
nothing on the next, whereas the id is whatever the backend that will place
this surface already uses.
