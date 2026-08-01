Whether a **queued request** is waiting for [`apply`](Self::apply).

A host polls this to decide whether an otherwise idle loop owes a turn: a
stylesheet replaced from a background callback has to reach the runtime even
though no input arrived.

Deliberately *not* about unresolved registrations. A name that matches nothing
stays pending forever, so a loop that woke for that would spin at the frame
rate for the rest of the application's life over one misspelled `test-id`. Ask
[`pending_bindings`](Self::pending_bindings) for those.
