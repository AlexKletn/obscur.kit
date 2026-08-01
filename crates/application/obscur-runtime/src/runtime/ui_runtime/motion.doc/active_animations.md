Returns the number of **running** animations — the count of active
timelines still driving motion.

This falls to zero once every timeline finishes (the scheduler prunes
finished timelines), even though bindings may linger to hold their final
value. It is the signal a host uses to decide whether to keep ticking
[`advance_motion`](Self::advance_motion).

The same reading as
[`active_animation_count`](Self::active_animation_count) — see its note.
