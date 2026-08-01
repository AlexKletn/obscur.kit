Something outside the compositor asked the loop to turn — a worker
thread, a timer, another protocol's connection — through a
[`LayerWaker`](crate::surface::LayerWaker).

It carries nothing, and cannot: the waker does not know why it was rung.
A host treats it as "reconsider the interface", which is what every
caller of a waker actually means.
