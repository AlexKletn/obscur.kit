Attaches a [`LayerHandle`](super::LayerHandle) the running loop publishes
to and reads requests from.

Build the handle first, clone it into whatever handlers need it, then give
one to this. It is how an application reconfigures the surface, reads its
real size and scale, and asks the loop to stop — none of which it can do
through a `&mut` it cannot have while the loop owns the surface.
