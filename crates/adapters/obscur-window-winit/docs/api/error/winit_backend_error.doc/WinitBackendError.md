A failure raised inside the winit window backend (spec §6.1).

This is the adapter-local error type for the platform boundary: it names the
concrete winit-side failures (event-loop and window construction, a missing
active event loop, and any other backend failure) while carrying only a
human-readable message so it stays `Clone`/`Eq` and free of borrowed winit
state. It is mapped into the domain's [`WindowError`] via
[`WinitBackendError::into_window_error`] before crossing back into the pure
windowing domain (§2.7).
