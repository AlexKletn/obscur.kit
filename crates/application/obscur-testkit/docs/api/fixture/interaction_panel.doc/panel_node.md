The container element itself, before it is wrapped in a `View`.

Separate from [`panel`](Self::panel) only because one caller has to keep building
on it: the bubbling arrangement binds a command to the container, and a `View` is
no longer a node one can add an event to.
