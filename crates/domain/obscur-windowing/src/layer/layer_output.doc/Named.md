A specific connector, by name.

A name that matches no connected output is **not** an error: the surface
falls back to the compositor's choice, because a monitor being unplugged
must not stop the panel from appearing at all. A host that needs to know
which it got asks the surface afterwards.
