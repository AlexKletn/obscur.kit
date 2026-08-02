§2.12: "- named slots;" (the partitioning half)

WEAKENED 2026-07-19: this originally also asserted that the two children
carrying no `slot=` assignment land in an unnamed DEFAULT slot, in written
order (`children(default).len() == 2`, classes `["d1", "d2"]`). §2.12's list
is "- named slots; - allowed child contracts; - slot cardinality; - exported
style parts; - exported visual states." — a default/unnamed slot is nowhere
in it, and the case's own `ViewNode::slot("")` outlet is an invention of the
test. The named-slot half, which the spec does state, is kept. Candidate spec
amendment: state whether a component may declare a default (unnamed) slot,
how it is spelled, and where unassigned children go when it is absent.

NOTE 2026-08-02: the *implementation* now has one, because the base-element
`<button>` could not be mounted without it — content that names no slot goes
to the outlet that names none, an outlet nothing fills renders its own
children as a fallback, and `ButtonProps::contract()` declares the slot under
the reserved `DEFAULT_SLOT` name. The behaviour is pinned where it belongs,
in `obscur-view`'s `slot` unit tests and in `tests/builtin_components.rs`.
This case is deliberately NOT restored: it is a *spec* case, and the spec
still does not state it. Amend §2.12 and it can be.
