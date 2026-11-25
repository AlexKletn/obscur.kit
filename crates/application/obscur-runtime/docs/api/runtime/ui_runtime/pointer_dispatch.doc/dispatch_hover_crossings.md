Dispatches `pointerenter` / `pointerleave` for the nodes this event's hover
transition crossed.

**Leave first, innermost-first; then enter, outermost-first.** The two sequences
nest: the old chain is unwound from the inside out before any of the new one is
entered from the outside in. That is the order the DOM fires its crossings in,
and the reason to keep it is not parity — it is that a handler pairing a
teardown on leave with a setup on enter is never interleaved with a sibling's,
even when the pointer crosses straight from one row into the next.

Nothing is dispatched unless the committed tree binds the name, and the lists
are empty unless the chain actually changed.

`prevent_default` has nothing to suppress on a crossing;
[`EVENT_POINTER_ENTER`](obscur_view::view::EVENT_POINTER_ENTER) says why.
