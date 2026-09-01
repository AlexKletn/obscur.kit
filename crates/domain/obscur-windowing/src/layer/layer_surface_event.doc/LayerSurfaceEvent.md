Something that happened to a desktop surface.

Deliberately the same shape as [`WindowEvent`](crate::window::WindowEvent), and
deliberately as small: **no input.** Pointer, touch and key events do not travel
this way for a window and do not travel this way for a layer surface either —
they reach the runtime through the interaction vocabulary, and a backend that
mixed the two would give a panel a different input path from every other
surface in the toolkit for no reason.

What is left is geometry, lifecycle, and the one event that has no platform
cause at all: [`Woken`](LayerSurfaceEvent::Woken).
