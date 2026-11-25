The rule describing which child components are allowed in a given position
(a slot, or a component's direct children) per spec §2.12.

This is a small, typed allow-list rather than a string matcher: unknown
children are rejected at contract-check time. The whitelist variant carries
the concrete [`ComponentId`]s that are permitted, so membership is decided
by identity, not by name.
