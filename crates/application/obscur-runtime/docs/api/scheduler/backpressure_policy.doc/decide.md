Decides how to admit an incoming request.

- If a request for the same reason is `already_queued`, the incoming one
  coalesces into it ([`BackpressureDecision::Coalesce`]).
- Otherwise, if the pending set is at or over the cap, the policy either
  drops a stale frame ([`BackpressureDecision::DropStale`]) when
  stale-skipping is enabled, or rejects the request
  ([`BackpressureDecision::Reject`]) when it is not.
- Otherwise the request is admitted ([`BackpressureDecision::Admit`]).
