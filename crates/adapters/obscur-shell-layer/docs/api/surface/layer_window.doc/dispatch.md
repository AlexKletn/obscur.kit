Turn the loop once, blocking until something happens or the timeout expires.

Produces nothing directly: what arrived is then taken by
[`poll_input`](Self::poll_input) and
[`LayerBackend::poll_events`](obscur_windowing::port::LayerBackend::poll_events).
One pump, two typed drains — the alternative, two pumps, is how events go
missing.

`None` sleeps until the compositor or a
[`LayerWaker`](crate::surface::LayerWaker) has something to say, which is where
a panel spends nearly all of its life.

# Errors

`Dispatch` if the Wayland connection failed while the loop was turning. There is
no recovery from it: the compositor is gone.
