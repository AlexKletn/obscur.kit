Raises one diagnostic per sibling group that reused a key (spec §2.4, §7.8).

Keyed identity assumes a key names one child. When several claim it the
matcher keeps the first and the rest find no partner, so they are inserted as
new while their predecessors are removed — every frame, which forces a full
restyle and a full layout and makes a list an order of magnitude slower. The
failure is otherwise completely silent: the UI still looks right, it is only
slow, so an author has no way to connect the symptom to the cause. This is the
diagnostic that connects it.

The recovery is [`RecoveryAction::Fallback`]: reconcile proceeds with the
first claimant and rebuilds the rest, so the frame is correct but not cheap.
