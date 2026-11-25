A live platform window paired with the domain [`WindowId`] it materializes
(spec §6.1, §6.2).

The winit backend is callback-driven: real [`winit::window::Window`]s can
only be created inside an
[`ActiveEventLoop`](winit::event_loop::ActiveEventLoop) callback, well after
`create_window` has already returned a [`WindowId`]. A `WinitWindow` is the
record the event-loop handler stores once the OS window finally exists, keyed
by the id the backend minted up front. The native window is held behind an
[`Arc`] so a frame scheduler or renderer can share it cheaply across threads
without reopening the OS handle.
