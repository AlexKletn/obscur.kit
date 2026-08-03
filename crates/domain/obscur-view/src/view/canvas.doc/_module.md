How a node names the painter that draws it.

The protocol lives here, in the domain, and not on the `Canvas` primitive,
because both ends of it are outside that crate: the primitive WRITES the
property and the runtime READS it, and the runtime cannot depend on a
components crate. `virtual_collection` binds a virtual list's row builder the
same way and for the same reason.

The symbol is a handle rather than a closure because a `ViewNode` is data —
cloneable, comparable, diffable — and a drawing callback is none of those.
