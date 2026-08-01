The winit-side mutable state shared between [`WinitWindowBackend`] and the
[`WinitAppHandler`] (spec §6.1).

This is the shared-queue half of the pull/callback bridge. The backend
handle enqueues work here — window creation, property reconciliation, focus,
and close requests — and the handler drains that work inside winit callbacks
(where an [`ActiveEventLoop`] and the live windows are reachable), pushing
converted [`WindowEvent`]s back for the backend to poll. It is deliberately
*not* where the domain [`Window`](obscur_windowing::window::Window) records
live: those are owned directly by the backend so `window()` can hand out a
`&Window` without borrowing through a mutex guard.

[`WinitWindowBackend`]: super::WinitWindowBackend
