Builds the process's one event loop, or `None` where there is no display.

`with_any_thread` is required, not a convenience: winit panics when an
event loop is built off the main thread ("Attempting to create the event
loop off the main thread will panic", `winit-0.30/src/event_loop.rs:90`),
and the Rust test harness runs every test on a spawned thread. The X11
extension trait is imported rather than the Wayland one for no reason
beyond having to pick: both are implemented for the same
`EventLoopBuilder<T>` and both assign the same backend-agnostic field —
`self.platform_specific.any_thread = any_thread` at `x11.rs:134` and
`wayland.rs:73`, where `any_thread: bool` is one field on the *shared*
linux `PlatformSpecificEventLoopAttributes` (`platform_impl/linux/mod.rs:57`),
not a per-backend one. So either import works under either session, and
importing both would only make the call ambiguous.
