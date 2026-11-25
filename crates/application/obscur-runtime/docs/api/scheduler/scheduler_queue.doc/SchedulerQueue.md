The queue of pending frame requests, coalesced by reason and ordered by
priority (spec §3.5 coalescing, §3.6 priority).

A [`FrameRequest`] carries only a [`FrameReason`], so coalescing means
deduplicating by reason: pushing two `Animation` requests collapses them into
a single pending entry (§3.5 "updates are coalesced"). Requests are drained
most-urgent-first using [`SchedulerPriority`] (§3.6), with ties broken
deterministically by a stable per-reason ordinal.

The ordering is backed by a [`BTreeSet`] keyed on
`(SchedulerPriority, reason ordinal)`, so iteration and popping are fully
deterministic and free of wall-clock or randomness.
