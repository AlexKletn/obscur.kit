The winit [`ApplicationHandler`] integration that bridges winit's
callback-driven event loop to the pull-based [`WindowBackend`] port
(spec §6.1 windowing; §2.7 "Реализует `WindowBackend` и преобразует native
events в доменные события"; §3.5 frame scheduling).

This module owns the three tightly-coupled pieces of that bridge: the
[`UserEvent`] the frame scheduler sends to wake the loop, the shared
[`BackendState`] queue both the backend handle and the handler mutate, and
the [`WinitAppHandler`] that drains it inside winit callbacks.
