§7.7, verbatim: "- enforces message/subscription limits;" and "- audits
dangerous operations;".

WEAKENED 2026-07-19, subscription-limit half only. It opened
`SUBSCRIPTION_LIMIT` subscriptions expecting each to be granted and the next
to be a typed `LimitExceeded`. Every subscription was denied instead, and
that denial is deliberate and documented: endpoint.rs:252-262 refuses a
`Subscribe` while `!self.event_push`, because "a grant is a promise to
deliver" and this session is request/reply only until a pushing loop marks
itself serviced. The case built exactly that environment and then asserted
against it.

It could not have built any other. `mark_event_push_serviced` is
`pub(crate)`, so no consumer outside `obscur-devtools-server` can put an
endpoint into the state where a subscription is grantable — which means
§7.7's "- enforces message/subscription limits;" has NO route to a test
through the public API. That is recorded as a finding here, not fixed:
implementation files are out of scope for this pass.

What replaces it is the §7.7-adjacent property the same code path does
expose, and it discriminates: an endpoint must not hand back a grant it has
no way to honour, and refusing must cost neither the connection nor an audit
entry. The "- audits dangerous operations;" half below is untouched.

Candidate spec amendment: say whether a subscription may be granted before a
delivery path exists, and expose whatever state gates that so the limit is
reachable from a conformance test.
