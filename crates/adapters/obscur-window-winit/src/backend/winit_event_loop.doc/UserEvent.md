A wake-up message sent to the winit event loop from any thread (spec §3.5).

The §3.5 frame scheduler runs off the event loop; when it decides a frame is
due it sends a `UserEvent` through an
[`EventLoopProxy`](winit::event_loop::EventLoopProxy) (wrapped by
[`EventLoopWaker`](crate::wake::EventLoopWaker)) to nudge the loop awake.
There is a single variant today, but the enum keeps the door open for
additional cross-thread signals without changing the event-loop type
parameter.
