Declares that the caller's loop drains an
[`EventQueue`](crate::transport::EventQueue) and writes this endpoint's
[`fan_out`](Self::fan_out) to the client, so subscriptions may be granted.

Deliberately crate-visible: the promise that `granted: true` makes is
only truthful if the code holding the endpoint really does push, and a
host outside this crate cannot push without going through a transport
here. That keeps the flag a fact rather than a claim.
