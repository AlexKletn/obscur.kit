Admits an incoming frame request through the backpressure policy (§3.5).

This is the event ingress: a request is a fact ("a frame is wanted for
this reason"), and this method decides whether it joins the pending set.
The [`BackpressurePolicy`] is consulted with whether the reason is already
queued and the current queue length, and the resulting
[`BackpressureDecision`] drives what happens:

- [`Admit`](BackpressureDecision::Admit) or
  [`DropStale`](BackpressureDecision::DropStale) → the request is pushed.
  Under [`DropStale`](BackpressureDecision::DropStale) a stale queued
  frame is allowed to be skipped later (§3.5 "stale intermediate frames
  may be skipped"); because the queue coalesces by reason, a fresher
  request for a reason already supersedes the stale one, so pushing is the
  correct realization of dropping the stale frame.
- [`Coalesce`](BackpressureDecision::Coalesce) → no-op; the reason is
  already pending and the incoming request folds into it (§3.5 "updates
  are coalesced").
- [`Reject`](BackpressureDecision::Reject) → the request is dropped; the
  pending set is full and stale-skipping is disabled.

Returns the [`BackpressureDecision`] so callers and diagnostics can
observe how the request was handled.
