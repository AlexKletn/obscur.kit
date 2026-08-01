§2.14: "- capability requirements;" (with §2.13 "- required capabilities;")

WEAKENED 2026-07-19: this originally asserted that instantiating a component
whose required capability has not been granted is REFUSED — factory not
invoked, no node created, and a diagnostic emitted. §2.14 says
"`ComponentRegistry` stores: … - capability requirements;" — *stores* — and
§2.13 likewise only lists "- required capabilities;" among what a contract
exports. Neither sentence says a requirement is enforced at instantiation.

RESTORED 2026-07-19 by the owner's decision, and the reasoning is worth
keeping. The literal reading above is defensible; the enforcement assertion
comes back anyway, because there IS no enforcement — the audit records that
`obscur-capability` is 189 lines and nothing consults a capability before
acting. Weakening this test would convert a known access-control gap from
visibly red to silently green, which is precisely what this suite exists to
prevent. The general ruling that the spec outranks a test was given on
examples about version-query APIs and template tags; it is not extended to a
capability boundary.

Candidate spec amendment, now load-bearing: state whether an ungranted
required capability blocks instantiation, which subsystem holds the grant
set, and what is emitted. §9 lists "weaken host control over plugin
capabilities" among the things the kit must not do, and §1.8 names
capability-based services as a condition of running untrusted code, so the
value is in the spec even where §2.13's enforcement sentence is not.

The observation moved to a bare `ComponentRegistry` — the subject §2.14 names
— because `UiRuntime` exposes no accessor for a registered component's
contract. Recorded as a finding; not fixed here, since implementation files
are out of scope for this pass.
