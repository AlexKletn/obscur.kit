Reserved symbol for the `automation-id` attribute — a node's stable identifier
for external automation and assistive tooling (spec §7.5).

# Why it is not `test-id`

§7.5 lists the two separately, and they answer to different owners. A `test-id`
belongs to the app's own test suite: it is churn-friendly, renamed whenever a
test is rewritten, and may be compiled out. An `automation-id` is a contract
with software outside the codebase — an automation harness or an accessibility
tool — which cannot be updated in the same commit that renames it.

Collapsing them into one attribute would force those two lifetimes onto one
string: either tests stop renaming freely, or external automation breaks when
they do. They share every other property (not a style attribute, no effect on
matching, layout or visuals) and the same `0x0000_02xx` namespace.
