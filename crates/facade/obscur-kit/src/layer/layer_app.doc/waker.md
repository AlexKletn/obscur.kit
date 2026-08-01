Attaches a [`LayerWaker`](crate::shell_layer::surface::LayerWaker) the
loop will answer to.

The loop sleeps on the compositor's socket, so nothing else can make it
turn — not a worker thread, not a timer, and not a second Wayland
connection belonging to some other protocol. Give it a waker and any of
them can.
