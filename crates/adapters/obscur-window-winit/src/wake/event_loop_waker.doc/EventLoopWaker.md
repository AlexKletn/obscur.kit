A cheap, cloneable handle that wakes the winit event loop from any thread to
request a frame (spec §3.5).

The §3.5 frame scheduler runs off the event loop and must be able to nudge it
awake when a frame becomes due — including from a timer thread. This wraps a
winit [`EventLoopProxy`], the one winit primitive that is `Send` and can post
into the loop from outside it. [`EventLoopWaker::wake`] sends a
[`UserEvent::WakeUp`], which the
[`WinitAppHandler`](crate::backend::WinitAppHandler) turns into redraw
requests. Being `Clone`, a waker can be handed to as many schedulers or
worker threads as needed.
