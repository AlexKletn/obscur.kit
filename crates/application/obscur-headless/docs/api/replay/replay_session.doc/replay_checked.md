Replays into `target` while checking each step against an `expected`
recording, flagging the first step whose recorded event differs.

Dispatch is identical to [`replay`](Self::replay); additionally, before
dispatching each step its event is compared to the event at the same
index in `expected`. The first mismatch (including a length mismatch)
sets the status to [`ReplayStatus::Diverged`] with that step index, but
replay still runs to the end so the tallies are complete.
