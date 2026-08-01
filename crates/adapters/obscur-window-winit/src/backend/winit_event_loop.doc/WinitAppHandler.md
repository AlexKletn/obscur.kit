The winit [`ApplicationHandler`] that services the shared [`BackendState`]
inside the event loop (spec §6.1, §2.7).

A host builds one via [`WinitWindowBackend::app_handler`] so it shares the
exact same [`BackendState`] as the backend handle, then hands it to
[`run_app`](winit::event_loop::EventLoop::run_app). On each turn the handler
materializes any deferred windows, applies queued property/focus/close
operations, and converts native window events into domain [`WindowEvent`]s.

[`WinitWindowBackend::app_handler`]: super::WinitWindowBackend::app_handler
