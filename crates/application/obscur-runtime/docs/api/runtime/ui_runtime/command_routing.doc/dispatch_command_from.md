Routes `command` down §2.6's chain starting at `origin` with **no device
payload**, gated by the command's registered [`CommandState`], requesting
an input frame when a link claimed it. Returns whether a handler ran.

`origin` is the node the command *came from*: the focused node for a
keyboard shortcut, the clicked node for a pointer activation. `None`
leaves only the non-positional links (window, application) — which is the
honest route when nothing is focused and nothing was clicked.

The envelope this builds reports [`EventOrigin::Synthetic`] and carries
[`EventPayload::Empty`](obscur_interaction::event::EventPayload::Empty),
because a caller reaching in through this path *is* a programmatic
dispatch (§7.4): there is no device state to report, and inventing a
pointer at the origin would hand a handler a position no finger was ever
at. A dispatch that does have a device behind it goes through
[`dispatch_command_event`](Self::dispatch_command_event) with that device's
event instead.
