The result of routing a pointer event through
[`UiRuntime::handle_pointer`](super::UiRuntime::handle_pointer).

It reports the node the pointer resolved to (the topmost hit, if any) and
whether this event actually moved the hover chain or the focus target —
the two signals a host uses to decide whether a restyle/repaint is warranted.
When both `hover_changed` and `focus_changed` are `false`, the runtime's
interaction state is byte-for-byte unchanged and no frame was requested.

It also carries the two §2.3 facts a host cannot reconstruct for itself:
[`current_target`](Self::current_target) — the node the fired handler is
*attached to*, which is not in general the node the pointer *hit* — and
[`trust`](Self::trust), whether the event was a real user gesture.
