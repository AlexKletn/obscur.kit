Letting pointer input pass through the surface to whatever is behind it.

What [`LayerPointerPolicy::Transparent`](crate::layer::LayerPointerPolicy::Transparent)
asks for. A platform that cannot express it leaves the surface clickable, and a
surface that only shows something then takes clicks it never uses — so the
answer is worth having before the surface is up rather than after a user
notices a dead patch of desktop.
