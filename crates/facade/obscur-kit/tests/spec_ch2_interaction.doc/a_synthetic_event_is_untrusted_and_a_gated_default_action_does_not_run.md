§2.3: "- trusted/untrusted origin."

WEAKENED then RESTORED 2026-07-19, by the owner's decision. A correction pass
dropped the gating assertion on the literal reading that §2.3's one bullet
requires the origin to be *reported* and says nothing about acting on it.
That reading is defensible and the assertion is restored anyway: the gate
genuinely does not exist, so weakening the test would turn a known
security-relevant gap from visibly red into silently green — which is the
exact defect the spec-derived suite was built to remove. A red test is the
honest state.

The general ruling ("where a test demands more than the spec's text says, the
spec wins") was given on examples about version-query APIs and template tags.
It was not given on a trust boundary, and is not extended to one here.

Candidate spec amendment, and it is now load-bearing rather than optional:
state whether any default action is gated on trust, and whether the gate
belongs to the runtime or the component. §9 protects "host control over
plugin capabilities" as a non-goal to weaken, and §1.8 names capability-based
services as a condition of running untrusted code — so the value this guards
is in the spec even though §2.3's enforcement sentence is not.
