§7.6 + §2.3 trust gate: an external client with an Inject token can push a
click over the socket, and the host dispatches it — it hit-tests the button
and moves interaction state — but because an injected event is UNTRUSTED, its
**gated default action** (§2.3 step 4: activating the control and firing its
bound command) does NOT run. An Inject-scoped agent drives *observation*, not
consequential actions; only a platform-vouched (trusted) gesture does that.

REWRITTEN 2026-07-21 (owner-authorized, non-spec). The prior version asserted
the injected click ran the button command (`clicked == true`). That encoded
the pre-enforcement assumption that an external inject acts. Under the now-real
trust gate — and consistent with this file's sibling
`injected_pointer_is_not_trusted_but_a_platform_event_is`, which still passes —
an untrusted inject is dispatched and observed but its default action is
suppressed. The untrusted-flag, acknowledgement, queue-drain and hit-test
assertions are unchanged; only the "command ran" expectation is inverted to
the enforced truth.
