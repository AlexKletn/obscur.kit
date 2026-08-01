§2.1: "- default error-boundary behavior." (with §1.7 "is contained within
the component subtree;" "replaces the subtree with fallback UI;")

SAMPLE POINT MOVED + WEAKENED 2026-07-19. Two things were wrong.

(a) The case read the tree one frame too early. §1.7's fourth consequence —
which the case itself cites — is "- may be retried according to component or
host policy.", and this runtime does retry: the failing frame is reported
with `recovery: Retried` and the subtree is not replaced until the attempt
after it. A second commit is now made before observing. The fallback
assertion is unchanged.

(b) It originally demanded EXACTLY ONE diagnostic. §1.7 says a failure "is
reported to diagnostics and DevTools" and separately that it "may be
retried"; nothing says a retried failure is reported once rather than once
per attempt, and §1.6 requires each diagnostic to carry a "recovery action"
— which is precisely what distinguishes the two here (`Retried`, then
`BoundaryReplaced`). The count is replaced by what the spec does require:
every report is attributed to the failing component's node and none to its
siblings. Candidate spec amendment: state whether a retried failure is
reported per attempt or coalesced, since DevTools noise depends on it.
