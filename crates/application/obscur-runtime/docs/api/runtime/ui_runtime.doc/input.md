The events awaiting the frame pipeline's dispatch phase, the results that
phase produced, and the frame requests it made while running.

One field rather than the four it replaces: the latch that redirects a
handler's own `request_frame` and the plan it redirects into are only
correct as a pair — see [`InputQueue`].
