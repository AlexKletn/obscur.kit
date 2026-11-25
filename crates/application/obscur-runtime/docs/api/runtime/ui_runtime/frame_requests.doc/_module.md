Everything that tells the runtime there is work to do: the frame ingress itself
([`request_frame`](UiRuntime::request_frame)), the three things that ask for a
frame on someone's behalf — a committed transaction, a notified signal — and
§3.6's background band, which is asked for and serviced through the same
scheduler.

The subject is the **scheduler's ingress**, not the frame: nothing here runs a
frame. `request_frame` is the one door, and its interesting half is that a
request raised *while the dispatch phase is running* is coalesced into the frame
that is running it rather than scheduled behind it (§3.5). `apply_transaction`
and `notify_signal` are that door reached through a commit and through
reactivity; each classifies what changed before it knocks, which is why a
transaction that only invalidated a style produces a frame that does not
reconcile.

Background work is here because §3.6 makes it a **band of the same scheduler**
rather than a second queue: `request_background` records a unit and tells the
scheduler one is waiting, and every higher-priority frame drained afterwards
counts as a preemption of it. `service_background_unit` is the other half of that
one bookkeeping — it is called from the frame entry points, before a frame is
pulled, once the scheduler reports the wait unfair. The readings over these same
counts are `pending_work.rs`'s.
