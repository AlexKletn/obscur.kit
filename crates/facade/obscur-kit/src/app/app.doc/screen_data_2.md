The table an `.okui` [`screen`](Self::screen)'s `{bindings}` read, as a handle
application code keeps.

Write it before `run` for the initial values and from a command handler for
every change afterwards; the next frame's lowering reads what is there. It is
plain application state — no runtime is involved — so a write is immediate and
needs no deferral, unlike [`control`](Self::control)'s requests.

Returns a clone of one shared table, so several handlers may each hold one.
