How a drag ended.

Returned by [`DragCoordinator::release`] so the caller can tell the two apart
without inspecting the session, and so the payload reaches the host on the one
path where it means something.
