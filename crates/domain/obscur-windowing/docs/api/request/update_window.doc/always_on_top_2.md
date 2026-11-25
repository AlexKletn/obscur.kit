Returns the requested always-on-top change, if any.

A backend inspects this to decide whether the delta touches a property it
cannot service (§6.2): keeping a window above others needs a compositor.
