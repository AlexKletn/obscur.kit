The backend's own event-loop handler, which this host wraps. Every winit
callback forwards to it first; it materializes the deferred window,
applies queued property work, and converts native events into the domain
events `poll_window_events` hands back.
