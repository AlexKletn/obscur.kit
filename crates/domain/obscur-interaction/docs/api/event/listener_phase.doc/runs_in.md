Returns whether a listener registered for this phase runs during `phase`.

At [`EventPhase::Target`] the answer is `true` for **both** variants: the
event has arrived at the node the listener is attached to, so the
direction it registered for no longer distinguishes anything. This mirrors
the DOM, where target-phase listeners run in registration order regardless
of their capture flag.
