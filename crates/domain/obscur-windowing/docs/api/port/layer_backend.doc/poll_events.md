Take everything that has happened since the last call.

Pull rather than push, matching
[`WindowBackend::poll_window_events`](crate::port::WindowBackend::poll_window_events):
the loop decides when to look, and a backend never calls into application code.
