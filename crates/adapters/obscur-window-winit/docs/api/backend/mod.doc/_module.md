The winit `WindowBackend` adapter and its event-loop integration (spec
§6.1-6.6 windowing; §2.7). [`WinitWindowBackend`] is the pull-based port
handle; [`WinitAppHandler`] services its shared queue inside winit's
callback-driven loop, converting native events (via
[`WindowEventConverter`](crate::conversion::WindowEventConverter)) into domain
events; [`WinitWindow`] pairs a domain id with a live winit window; and
[`UserEvent`] is the cross-thread wake-up the frame scheduler sends.
