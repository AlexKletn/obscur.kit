§5.3's canvas, reached the way an application reaches it.

`Canvas` had a `ComponentId` and nothing in the runtime or either adapter ever
named it: it laid out, clipped and hit-tested like any element, and could not
draw. These cases drive the whole road — an application registers a painter, a
node names it, and what the painter draws lands in the scene inside its own box.
