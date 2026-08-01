Stable, opaque identity for a [`Timeline`](super::Timeline) (spec §4.17
"stable identity").

The motion engine must keep a timeline recognisable across interruption,
reversal, chaining and re-scheduling, so every timeline carries a stable
`TimelineId` that never changes for its lifetime. This is a *local* newtype:
the motion crate mints and compares these handles itself and does not depend
on any shared identity crate. Backed by an integer handle; the numeric value
carries no meaning outside the subsystem that mints it.
