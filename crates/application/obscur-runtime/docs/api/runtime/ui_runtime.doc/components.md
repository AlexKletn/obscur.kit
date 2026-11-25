The registered component kinds and one live instance per component-backed
node (spec §2.1, §2.14).

Empty and inert until a host calls
[`register_component`](Self::register_component): with no registrations
nothing mounts, so a host that drives the tree purely from its root view
builder pays nothing for this.
