A mock external engine that owns the window, the event loop, the render graph,
the lifecycle and the frame callback, and hands the kit only what §5.4 says
the kit receives: viewport, input, clock, render target, and frame scheduling
hooks.

It deliberately exposes no way to create a window and never enters a loop:
[`windows_created`](Self::windows_created) and
[`event_loop_entered`](Self::event_loop_entered) are the observations the
§5.4 cases assert on.
